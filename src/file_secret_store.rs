/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::HashMap;
use std::fmt::Display;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};

use crate::error::LkError;
use crate::secrets_manager::{SecretStore, SECRETSFILE_PREFIX};
use crate::utils::strip_unprintable;

const STORE_FILE_NAME: &str = "secrets.enc";
const KEY_FILE_NAME: &str = "secrets.key";
const KEY_FILE_ENV: &str = "LIGHTKEEPER_SECRETS_KEY_FILE";
const STORE_VERSION: u32 = 1;
const AAD: &[u8] = b"lightkeeper-core-secrets-v1";
const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 24;

type Entries = HashMap<String, String>;

#[derive(Serialize, Deserialize)]
struct EncryptedStoreFile {
    version: u32,
    nonce: String,
    ciphertext: String,
}

pub struct FileSecretStore {
    store_path: PathBuf,
    key_path: PathBuf,
    /// Only the default `<config-dir>/secrets.key` may be created automatically.
    allow_generate_key: bool,
    /// Also serializes all store file access.
    key: Mutex<Option<Key>>,
}

impl FileSecretStore {
    /// Opens a store under `data_dir` with the wrapping key under `config_dir` by default.
    /// An explicit key file (argument or environment) overrides the default key path.
    pub fn open(data_dir: PathBuf, config_dir: PathBuf, key_file_override: Option<PathBuf>) -> Result<Self, LkError> {
        fs::create_dir_all(&data_dir).map_err(context(format!("Failed to create data directory {}", data_dir.display())))?;
        fs::create_dir_all(&config_dir)
            .map_err(context(format!("Failed to create config directory {}", config_dir.display())))?;

        let store_path = data_dir.join(STORE_FILE_NAME);
        let explicit_key_path = key_file_override
            .or_else(|| std::env::var(KEY_FILE_ENV).ok().filter(|path| !path.is_empty()).map(PathBuf::from));
        let allow_generate_key = explicit_key_path.is_none();
        let key_path = explicit_key_path.unwrap_or_else(|| config_dir.join(KEY_FILE_NAME));

        let key = if key_path.exists() { Some(read_key_file(&key_path)?) } else { None };

        match (&key, allow_generate_key) {
            (Some(_), _) => log::info!("Using encrypted file secret store at {}", store_path.display()),
            (None, _) if store_path.exists() => log::warn!(
                "Secret key not found at {} but {} exists; secrets unavailable until the key is provided",
                key_path.display(),
                store_path.display()
            ),
            (None, true) => log::info!(
                "Using encrypted file secret store at {} (key will be created on first write)",
                store_path.display()
            ),
            (None, false) => log::info!(
                "Using encrypted file secret store at {} (waiting for key at {})",
                store_path.display(),
                key_path.display()
            ),
        }

        Ok(Self { store_path, key_path, allow_generate_key, key: Mutex::new(key) })
    }

    fn modify_entries(&self, modify: impl FnOnce(&mut Entries)) -> Result<(), LkError> {
        let mut key = self.key.lock()?;
        let mut entries = self.read_entries(&mut key)?;
        modify(&mut entries);
        let key = self.ensure_key(&mut key)?;
        self.write_entries(&key, &entries)
    }

    /// Loads the key lazily so that a key file provided after startup is picked up.
    fn read_entries(&self, key: &mut Option<Key>) -> Result<Entries, LkError> {
        if key.is_none() && self.key_path.exists() {
            *key = Some(read_key_file(&self.key_path)?);
        }
        if !self.store_path.exists() {
            return Ok(Entries::new());
        }
        let key = key.as_ref().ok_or_else(|| {
            LkError::other(format!(
                "Secret key not found at {} but {} exists; secrets unavailable",
                self.key_path.display(),
                self.store_path.display()
            ))
        })?;

        let raw = fs::read(&self.store_path).map_err(context(format!("Failed to read secret store {}", self.store_path.display())))?;
        let envelope: EncryptedStoreFile = serde_json::from_slice(&raw).map_err(context("Invalid secret store format"))?;
        if envelope.version != STORE_VERSION {
            return Err(LkError::other(format!("Unsupported secret store version {}", envelope.version)));
        }

        let nonce = BASE64.decode(envelope.nonce.as_bytes()).map_err(context("Invalid secret store nonce"))?;
        if nonce.len() != NONCE_LEN {
            return Err(LkError::other("Invalid secret store nonce length"));
        }
        let ciphertext = BASE64.decode(envelope.ciphertext.as_bytes()).map_err(context("Invalid secret store ciphertext"))?;

        let plaintext = XChaCha20Poly1305::new(key)
            .decrypt(XNonce::from_slice(&nonce), Payload { msg: &ciphertext, aad: AAD })
            .map_err(|_| {
                LkError::other(
                    "Failed to decrypt secret store (wrong key or tampered file). \
                     Do not regenerate the key while secrets.enc exists."
                )
            })?;

        serde_json::from_slice(&plaintext).map_err(context("Invalid decrypted secret store payload"))
    }

    fn write_entries(&self, key: &Key, entries: &Entries) -> Result<(), LkError> {
        let plaintext = serde_json::to_vec(entries).map_err(context("Failed to serialize secret store"))?;
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = XChaCha20Poly1305::new(key)
            .encrypt(&nonce, Payload { msg: &plaintext, aad: AAD })
            .map_err(|_| LkError::other("Failed to encrypt secret store"))?;

        let envelope = EncryptedStoreFile {
            version: STORE_VERSION,
            nonce: BASE64.encode(nonce),
            ciphertext: BASE64.encode(&ciphertext),
        };
        let raw = serde_json::to_vec(&envelope).map_err(context("Failed to serialize encrypted secret store"))?;
        atomic_write(&self.store_path, &raw)
    }

    fn ensure_key(&self, key: &mut Option<Key>) -> Result<Key, LkError> {
        if let Some(key) = key {
            return Ok(*key);
        }
        if self.store_path.exists() {
            return Err(LkError::other(format!(
                "Secret key not found at {} but {} exists; refusing to create a new key",
                self.key_path.display(),
                self.store_path.display()
            )));
        }
        if !self.allow_generate_key {
            return Err(LkError::other(format!(
                "Secrets key file {} does not exist; create it or omit --secrets-key-file / {} to auto-generate",
                self.key_path.display(),
                KEY_FILE_ENV
            )));
        }

        let new_key = XChaCha20Poly1305::generate_key(&mut OsRng);
        if let Some(parent) = self.key_path.parent() {
            fs::create_dir_all(parent).map_err(context(format!("Failed to create key directory {}", parent.display())))?;
        }
        atomic_write(&self.key_path, BASE64.encode(new_key).as_bytes())?;
        log::warn!(
            "Generated secret store key at {}. Back it up separately from config; losing it loses all stored secrets.",
            self.key_path.display()
        );
        *key = Some(new_key);
        Ok(new_key)
    }
}

impl SecretStore for FileSecretStore {
    fn identifier_prefix(&self) -> &'static str {
        SECRETSFILE_PREFIX
    }

    fn get(&self, key: &str) -> Result<Option<String>, LkError> {
        let mut store_key = self.key.lock()?;
        Ok(self.read_entries(&mut store_key)?.get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), LkError> {
        let value = strip_unprintable(value);
        self.modify_entries(|entries| {
            entries.insert(key.to_string(), value);
        })
    }

    fn delete(&self, key: &str) -> Result<(), LkError> {
        if !self.store_path.exists() {
            return Ok(());
        }
        self.modify_entries(|entries| {
            entries.remove(key);
        })
    }
}

fn context<E: Display>(message: impl Display) -> impl FnOnce(E) -> LkError {
    move |error| LkError::other(format!("{}: {}", message, error))
}

fn read_key_file(path: &Path) -> Result<Key, LkError> {
    let raw = fs::read(path).map_err(context(format!("Failed to read secret key {}", path.display())))?;
    let decoded = BASE64.decode(raw.trim_ascii()).map_err(context(format!("Failed to decode secret key {}", path.display())))?;
    if decoded.len() != KEY_LEN {
        return Err(LkError::other(format!(
            "Secret key {} must be base64 of {} bytes",
            path.display(),
            KEY_LEN
        )));
    }
    Ok(*Key::from_slice(&decoded))
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), LkError> {
    let temp_path = path.with_extension("tmp");
    // Stale temp files are removed so that mode 0o600 always applies and symlinks aren't followed.
    match fs::remove_file(&temp_path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(LkError::other(format!("Failed to remove {}: {}", temp_path.display(), error)));
        },
        _ => {},
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp_path)
        .map_err(context(format!("Failed to create {}", temp_path.display())))?;
    file.write_all(contents).map_err(context(format!("Failed to write {}", temp_path.display())))?;
    file.sync_all().map_err(context(format!("Failed to sync {}", temp_path.display())))?;
    drop(file);

    fs::rename(&temp_path, path).map_err(context(format!("Failed to replace {} with {}", path.display(), temp_path.display())))?;
    // Directory sync makes the rename durable, so the key can't be lost while the store survives.
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(context(format!("Failed to sync directory {}", parent.display())))
}
