/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::ops::RangeInclusive;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::error::LkError;
use crate::secrets_manager::{SecretStore, SecretStoreStatus, SECRETSFILE_PREFIX};
use crate::utils::strip_unprintable;

const STORE_FILE_NAME: &str = "secrets.enc";
const PENDING_STORE_FILE_NAME: &str = "secrets.enc.pending";
const KEY_FILE_NAME: &str = "secrets.key";
const KEY_FILE_ENV: &str = "LIGHTKEEPER_SECRETS_KEY_FILE";
const STORE_VERSION: u32 = 1;
const AAD: &[u8] = b"lightkeeper-core-secrets-v1";
const KEY_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 24;
const PROTECTED_KEY_FORMAT: &str = "lightkeeper-secrets-key";
const PROTECTED_KEY_VERSION: u32 = 1;
const PROTECTED_KEY_KDF: &str = "argon2id";
const SALT_LENGTH: usize = 16;
const PASSPHRASE_LENGTH: RangeInclusive<usize> = 8..=1024;
/// Minimums prevent weakening a re-wrapped key via a doctored file, maximums prevent memory and CPU exhaustion.
const KDF_M_COST_KIB: RangeInclusive<u32> = 19 * 1024..=1024 * 1024;
const KDF_T_COST: RangeInclusive<u32> = 2..=10;
const KDF_P_COST: RangeInclusive<u32> = 1..=8;
/// About 1-2 s per attempt on a modern CPU, while keeping memory use modest for small hosts and containers.
const DEFAULT_KDF_M_COST_KIB: u32 = 256 * 1024;
const DEFAULT_KDF_T_COST: u32 = 6;
const DEFAULT_KDF_P_COST: u32 = 1;

type Entries = HashMap<String, String>;

#[derive(Serialize, Deserialize)]
struct EncryptedStoreFile {
    version: u32,
    nonce: String,
    ciphertext: String,
}

/// Passphrase-protected `secrets.key`. Argon2id derives a wrapping key that encrypts the store key.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtectedKeyFile {
    format: String,
    version: u32,
    kdf: String,
    m_cost: u32,
    t_cost: u32,
    p_cost: u32,
    salt: String,
    nonce: String,
    wrapped_key: String,
}

impl ProtectedKeyFile {
    /// Binds all header fields to the wrapped key so that any modification fails decryption.
    fn associated_data(&self) -> Vec<u8> {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.format, self.version, self.kdf, self.m_cost, self.t_cost, self.p_cost, self.salt
        )
        .into_bytes()
    }

    /// Validates the header before running the KDF so that hostile parameters are rejected early.
    fn derive_wrapping_key(&self, passphrase: &str) -> Result<Zeroizing<[u8; KEY_LENGTH]>, LkError> {
        if self.format != PROTECTED_KEY_FORMAT || self.version != PROTECTED_KEY_VERSION || self.kdf != PROTECTED_KEY_KDF {
            return Err(LkError::other("Unsupported protected secret key format"));
        }
        if !KDF_M_COST_KIB.contains(&self.m_cost) || !KDF_T_COST.contains(&self.t_cost) || !KDF_P_COST.contains(&self.p_cost) {
            return Err(LkError::other("Protected secret key has KDF parameters outside of allowed limits"));
        }
        if passphrase.len() > *PASSPHRASE_LENGTH.end() {
            return Err(LkError::other("Passphrase is too long"));
        }
        let salt = BASE64
            .decode(self.salt.as_bytes())
            .map_err(|error| LkError::other_p("Invalid protected secret key salt", error))?;
        if salt.len() != SALT_LENGTH {
            return Err(LkError::other("Invalid protected secret key salt length"));
        }

        let params = Params::new(self.m_cost, self.t_cost, self.p_cost, Some(KEY_LENGTH))
            .map_err(|error| LkError::other_p("Invalid KDF parameters", error))?;
        let mut wrapping_key = Zeroizing::new([0u8; KEY_LENGTH]);
        Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
            .hash_password_into(passphrase.as_bytes(), &salt, wrapping_key.as_mut_slice())
            .map_err(|error| LkError::other_p("Failed to derive wrapping key", error))?;
        Ok(wrapping_key)
    }

    fn wrap_key(key: &Key, passphrase: &str) -> Result<Self, LkError> {
        if !PASSPHRASE_LENGTH.contains(&passphrase.len()) {
            return Err(LkError::other(format!("Passphrase must be {}-{} bytes", PASSPHRASE_LENGTH.start(), PASSPHRASE_LENGTH.end())));
        }
        let mut salt = [0u8; SALT_LENGTH];
        OsRng.fill_bytes(&mut salt);
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let mut file = Self {
            format: PROTECTED_KEY_FORMAT.to_string(),
            version: PROTECTED_KEY_VERSION,
            kdf: PROTECTED_KEY_KDF.to_string(),
            m_cost: DEFAULT_KDF_M_COST_KIB,
            t_cost: DEFAULT_KDF_T_COST,
            p_cost: DEFAULT_KDF_P_COST,
            salt: BASE64.encode(salt),
            nonce: BASE64.encode(nonce),
            wrapped_key: String::new(),
        };

        let wrapping_key = file.derive_wrapping_key(passphrase)?;
        let wrapped_key = XChaCha20Poly1305::new(Key::from_slice(wrapping_key.as_slice()))
            .encrypt(&nonce, Payload { msg: key, aad: &file.associated_data() })
            .map_err(|_| LkError::other("Failed to wrap secret key"))?;
        file.wrapped_key = BASE64.encode(wrapped_key);
        Ok(file)
    }

    fn unwrap_key(&self, passphrase: &str) -> Result<Key, LkError> {
        let wrapping_key = self.derive_wrapping_key(passphrase)?;
        let nonce = BASE64
            .decode(self.nonce.as_bytes())
            .map_err(|error| LkError::other_p("Invalid protected secret key nonce", error))?;
        if nonce.len() != NONCE_LENGTH {
            return Err(LkError::other("Invalid protected secret key nonce length"));
        }
        let wrapped_key = BASE64
            .decode(self.wrapped_key.as_bytes())
            .map_err(|error| LkError::other_p("Invalid protected secret key", error))?;

        let associated_data = self.associated_data();
        let payload = Payload { msg: &wrapped_key, aad: &associated_data };
        let key = Zeroizing::new(
            XChaCha20Poly1305::new(Key::from_slice(wrapping_key.as_slice()))
                .decrypt(XNonce::from_slice(&nonce), payload)
                .map_err(|_| LkError::other("Failed to unlock secret key (wrong passphrase or tampered file)"))?,
        );
        if key.len() != KEY_LENGTH {
            return Err(LkError::other("Invalid unwrapped secret key length"));
        }
        Ok(*Key::from_slice(&key))
    }
}

enum KeyFile {
    Raw(Key),
    Protected(ProtectedKeyFile),
}

enum KeyState {
    NoKey,
    Unlocked { key: Key, protected: bool },
    Locked,
}

impl From<KeyFile> for KeyState {
    fn from(key_file: KeyFile) -> Self {
        match key_file {
            KeyFile::Raw(key) => KeyState::Unlocked { key, protected: false },
            KeyFile::Protected(_) => KeyState::Locked,
        }
    }
}

pub struct FileSecretStore {
    store_path: PathBuf,
    /// Store re-encrypted with a rotated key. Replaces the store only after the new key file has been written.
    pending_store_path: PathBuf,
    key_path: PathBuf,
    /// Only the default `<config-dir>/secrets.key` may be created automatically.
    allow_generate_key: bool,
    /// Also serializes all store file access and unlocking.
    state: Mutex<KeyState>,
}

impl FileSecretStore {
    /// Opens a store under `data_dir` with the wrapping key under `config_dir` by default.
    /// An explicit key file (argument or environment) overrides the default key path.
    pub fn open(data_dir: PathBuf, config_dir: PathBuf, key_file_override: Option<PathBuf>) -> Result<Self, LkError> {
        fs::create_dir_all(&data_dir)
            .map_err(|error| LkError::other(format!("Failed to create data directory {}: {}", data_dir.display(), error)))?;
        fs::create_dir_all(&config_dir)
            .map_err(|error| LkError::other(format!("Failed to create config directory {}: {}", config_dir.display(), error)))?;

        let store_path = data_dir.join(STORE_FILE_NAME);
        let explicit_key_path =
            key_file_override.or_else(|| std::env::var(KEY_FILE_ENV).ok().filter(|path| !path.is_empty()).map(PathBuf::from));
        let allow_generate_key = explicit_key_path.is_none();
        let key_path = explicit_key_path.unwrap_or_else(|| config_dir.join(KEY_FILE_NAME));

        let state = if key_path.exists() { KeyState::from(read_key_file(&key_path)?) } else { KeyState::NoKey };

        match (&state, allow_generate_key) {
            (KeyState::Unlocked { .. }, _) => log::info!("Using encrypted file secret store at {}", store_path.display()),
            (KeyState::Locked, _) => log::info!(
                "Using encrypted file secret store at {} (locked until unlocked with passphrase)",
                store_path.display()
            ),
            (KeyState::NoKey, _) if store_path.exists() => log::warn!(
                "Secret key not found at {} but {} exists; secrets unavailable until the key is provided",
                key_path.display(),
                store_path.display()
            ),
            (KeyState::NoKey, true) => log::info!(
                "Using encrypted file secret store at {} (key will be created on first write)",
                store_path.display()
            ),
            (KeyState::NoKey, false) => log::info!(
                "Using encrypted file secret store at {} (waiting for key at {})",
                store_path.display(),
                key_path.display()
            ),
        }

        Ok(Self {
            pending_store_path: data_dir.join(PENDING_STORE_FILE_NAME),
            store_path,
            key_path,
            allow_generate_key,
            state: Mutex::new(state),
        })
    }

    /// Wrong passphrase and corrupt key file result in the same error.
    fn unwrap_key_file(&self, passphrase: &str) -> Result<Key, LkError> {
        let KeyFile::Protected(protected) = read_key_file(&self.key_path)? else {
            return Err(LkError::other(format!("Secret key {} is not passphrase-protected", self.key_path.display())));
        };
        protected.unwrap_key(passphrase).map_err(|error| {
            log::debug!("Unlocking secret key failed: {}", error);
            LkError::other("Failed to unlock secret key")
        })
    }

    fn modify_entries(&self, modify: impl FnOnce(&mut Entries)) -> Result<(), LkError> {
        let mut state = self.state.lock()?;
        let mut entries = self.read_entries(&mut state)?;
        modify(&mut entries);
        let key = self.ensure_key(&mut state)?;
        write_store_file(&self.store_path, &key, &entries)
    }

    /// Loads the key lazily so that a key file provided after startup is picked up.
    fn read_entries(&self, state: &mut KeyState) -> Result<Entries, LkError> {
        if matches!(state, KeyState::NoKey) && self.key_path.exists() {
            *state = KeyState::from(read_key_file(&self.key_path)?);
        }
        let key = match state {
            KeyState::Locked => return Err(LkError::other("Secret store is locked")),
            _ if !self.store_path.exists() => return Ok(Entries::new()),
            KeyState::Unlocked { key, .. } => key,
            KeyState::NoKey => {
                return Err(LkError::other(format!(
                    "Secret key not found at {} but {} exists; secrets unavailable",
                    self.key_path.display(),
                    self.store_path.display()
                )))
            }
        };

        match read_store_file(&self.store_path, key) {
            Ok(entries) => {
                if self.pending_store_path.exists() {
                    fs::remove_file(&self.pending_store_path).map_err(|error| {
                        LkError::other(format!("Failed to remove {}: {}", self.pending_store_path.display(), error))
                    })?;
                }
                Ok(entries)
            }
            Err(error) if self.pending_store_path.exists() => {
                let entries = read_store_file(&self.pending_store_path, key).map_err(|_| error)?;
                fs::rename(&self.pending_store_path, &self.store_path)
                    .map_err(|error| LkError::other(format!("Failed to replace {}: {}", self.store_path.display(), error)))?;
                log::warn!("Recovered secret store from interrupted key rotation");
                Ok(entries)
            }
            Err(error) => Err(error),
        }
    }

    fn ensure_key(&self, state: &mut KeyState) -> Result<Key, LkError> {
        match state {
            KeyState::Unlocked { key, .. } => return Ok(*key),
            KeyState::Locked => return Err(LkError::other("Secret store is locked")),
            KeyState::NoKey => {}
        }
        if self.key_path.exists() {
            return Err(LkError::other(format!("Refusing to overwrite existing secret key {}", self.key_path.display())));
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
            fs::create_dir_all(parent)
                .map_err(|error| LkError::other(format!("Failed to create key directory {}: {}", parent.display(), error)))?;
        }
        atomic_write(&self.key_path, BASE64.encode(new_key).as_bytes())?;
        log::warn!(
            "Generated secret store key at {}. Back it up separately from config; losing it loses all stored secrets.",
            self.key_path.display()
        );
        *state = KeyState::Unlocked { key: new_key, protected: false };
        Ok(new_key)
    }
}

impl SecretStore for FileSecretStore {
    fn identifier_prefix(&self) -> &'static str {
        SECRETSFILE_PREFIX
    }

    fn get(&self, key: &str) -> Result<Option<String>, LkError> {
        let mut state = self.state.lock()?;
        Ok(self.read_entries(&mut state)?.get(key).cloned())
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

    fn status(&self) -> Result<SecretStoreStatus, LkError> {
        let mut state = self.state.lock()?;
        if matches!(*state, KeyState::NoKey) && self.key_path.exists() {
            *state = KeyState::from(read_key_file(&self.key_path)?);
        }
        Ok(match *state {
            KeyState::NoKey => SecretStoreStatus::NoKey,
            KeyState::Unlocked { protected: false, .. } => SecretStoreStatus::Unlocked,
            KeyState::Unlocked { protected: true, .. } => SecretStoreStatus::UnlockedWithPassphrase,
            KeyState::Locked => SecretStoreStatus::Locked,
        })
    }

    /// Unlocks a passphrase-protected key and keeps it in memory until exit.
    fn unlock_store(&self, passphrase: &str) -> Result<(), LkError> {
        let mut state = self.state.lock()?;
        if !matches!(*state, KeyState::Locked) {
            return Ok(());
        }
        let key = self.unwrap_key_file(passphrase)?;
        *state = KeyState::Unlocked { key, protected: true };
        log::info!("Unlocked secret store key {}", self.key_path.display());
        Ok(())
    }

    /// Protects a raw key with a passphrase. The key is rotated since the old raw key may exist in backups.
    fn protect_store(&self, passphrase: &str) -> Result<(), LkError> {
        let mut state = self.state.lock()?;
        let entries = self.read_entries(&mut state)?;
        self.ensure_key(&mut state)?;
        if matches!(*state, KeyState::Unlocked { protected: true, .. }) {
            return Err(LkError::other("Secret key is already passphrase-protected"));
        }

        let new_key = XChaCha20Poly1305::generate_key(&mut OsRng);
        let protected = ProtectedKeyFile::wrap_key(&new_key, passphrase)?;
        let protected_raw = serde_json::to_vec_pretty(&protected)
            .map_err(|error| LkError::other_p("Failed to serialize protected secret key", error))?;
        // A crash between these writes is recovered in `read_entries` by promoting the pending store.
        write_store_file(&self.pending_store_path, &new_key, &entries)?;
        atomic_write(&self.key_path, &protected_raw)?;
        *state = KeyState::Unlocked { key: new_key, protected: true };
        fs::rename(&self.pending_store_path, &self.store_path)
            .map_err(|error| LkError::other(format!("Failed to replace {}: {}", self.store_path.display(), error)))?;
        log::info!("Secret key {} is now passphrase-protected", self.key_path.display());
        Ok(())
    }

    /// Requires the current passphrase even when unlocked. Only re-wraps the key.
    fn change_store_passphrase(&self, old_passphrase: &str, new_passphrase: &str) -> Result<(), LkError> {
        let mut state = self.state.lock()?;
        let key = self.unwrap_key_file(old_passphrase)?;
        let protected = ProtectedKeyFile::wrap_key(&key, new_passphrase)?;
        let protected_raw = serde_json::to_vec_pretty(&protected)
            .map_err(|error| LkError::other_p("Failed to serialize protected secret key", error))?;
        atomic_write(&self.key_path, &protected_raw)?;
        *state = KeyState::Unlocked { key, protected: true };
        log::info!("Changed passphrase of secret key {}", self.key_path.display());
        Ok(())
    }

    /// Requires the current passphrase. Confirmation is the caller's responsibility.
    fn unprotect_store(&self, passphrase: &str) -> Result<(), LkError> {
        let mut state = self.state.lock()?;
        let key = self.unwrap_key_file(passphrase)?;
        atomic_write(&self.key_path, BASE64.encode(key).as_bytes())?;
        *state = KeyState::Unlocked { key, protected: false };
        log::warn!("Removed passphrase from secret key {}; the key is now stored unencrypted", self.key_path.display());
        Ok(())
    }
}

fn read_store_file(path: &Path, key: &Key) -> Result<Entries, LkError> {
    let raw = fs::read(path)
        .map_err(|error| LkError::other(format!("Failed to read secret store {}: {}", path.display(), error)))?;
    let envelope: EncryptedStoreFile = serde_json::from_slice(&raw)
        .map_err(|error| LkError::other_p("Invalid secret store format", error))?;
    if envelope.version != STORE_VERSION {
        return Err(LkError::other(format!("Unsupported secret store version {}", envelope.version)));
    }

    let nonce = BASE64
        .decode(envelope.nonce.as_bytes())
        .map_err(|error| LkError::other_p("Invalid secret store nonce", error))?;
    if nonce.len() != NONCE_LENGTH {
        return Err(LkError::other("Invalid secret store nonce length"));
    }
    let ciphertext = BASE64
        .decode(envelope.ciphertext.as_bytes())
        .map_err(|error| LkError::other_p("Invalid secret store ciphertext", error))?;

    let plaintext = XChaCha20Poly1305::new(key)
        .decrypt(XNonce::from_slice(&nonce), Payload { msg: &ciphertext, aad: AAD })
        .map_err(|_| {
            LkError::other(
                "Failed to decrypt secret store (wrong key or tampered file). \
                 Do not regenerate the key while secrets.enc exists.",
            )
        })?;

    serde_json::from_slice(&plaintext)
        .map_err(|error| LkError::other_p("Invalid decrypted secret store payload", error))
}

fn write_store_file(path: &Path, key: &Key, entries: &Entries) -> Result<(), LkError> {
    let plaintext = serde_json::to_vec(entries)
        .map_err(|error| LkError::other_p("Failed to serialize secret store", error))?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = XChaCha20Poly1305::new(key)
        .encrypt(&nonce, Payload { msg: &plaintext, aad: AAD })
        .map_err(|_| LkError::other("Failed to encrypt secret store"))?;

    let envelope = EncryptedStoreFile {
        version: STORE_VERSION,
        nonce: BASE64.encode(nonce),
        ciphertext: BASE64.encode(&ciphertext),
    };
    let raw = serde_json::to_vec(&envelope)
        .map_err(|error| LkError::other_p("Failed to serialize encrypted secret store", error))?;
    atomic_write(path, &raw)
}

fn read_key_file(path: &Path) -> Result<KeyFile, LkError> {
    let raw = Zeroizing::new(
        fs::read(path).map_err(|error| LkError::other(format!("Failed to read secret key {}: {}", path.display(), error)))?,
    );
    // Raw keys are base64 and can't start with '{', so a malformed protected key never falls back to a raw key.
    if raw.trim_ascii().starts_with(b"{") {
        let protected = serde_json::from_slice(&raw)
            .map_err(|error| LkError::other(format!("Invalid protected secret key {}: {}", path.display(), error)))?;
        return Ok(KeyFile::Protected(protected));
    }
    let decoded = Zeroizing::new(
        BASE64
            .decode(raw.trim_ascii())
            .map_err(|error| LkError::other(format!("Failed to decode secret key {}: {}", path.display(), error)))?,
    );
    if decoded.len() != KEY_LENGTH {
        return Err(LkError::other(format!(
            "Secret key {} must be base64 of {} bytes",
            path.display(),
            KEY_LENGTH
        )));
    }
    Ok(KeyFile::Raw(*Key::from_slice(&decoded)))
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), LkError> {
    let temp_path = path.with_extension("tmp");
    // Stale temp files are removed so that mode 0o600 always applies and symlinks aren't followed.
    match fs::remove_file(&temp_path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(LkError::other(format!("Failed to remove {}: {}", temp_path.display(), error)));
        }
        _ => {}
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temp_path)
        .map_err(|error| LkError::other(format!("Failed to create {}: {}", temp_path.display(), error)))?;
    file.write_all(contents)
        .map_err(|error| LkError::other(format!("Failed to write {}: {}", temp_path.display(), error)))?;
    file.sync_all()
        .map_err(|error| LkError::other(format!("Failed to sync {}: {}", temp_path.display(), error)))?;
    drop(file);

    fs::rename(&temp_path, path)
        .map_err(|error| LkError::other(format!("Failed to replace {} with {}: {}", path.display(), temp_path.display(), error)))?;
    // Directory sync makes the rename durable, so the key can't be lost while the store survives.
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| LkError::other(format!("Failed to sync directory {}: {}", parent.display(), error)))
}
