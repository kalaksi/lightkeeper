/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use lightkeeper::file_secret_store::FileSecretStore;
use lightkeeper::secrets_manager::{set_store_passphrase, SecretStore, SecretStoreStatus, SECRETSFILE_PREFIX};

use crate::{unique_target_dir, EnvVarGuard};

struct Fixture {
    store: FileSecretStore,
    data_dir: PathBuf,
    config_dir: PathBuf,
    key_path: PathBuf,
    store_path: PathBuf,
    pending_path: PathBuf,
    _clear_key_env: EnvVarGuard,
}

impl Fixture {
    fn new() -> Self {
        let clear_key_env = EnvVarGuard::remove("LIGHTKEEPER_SECRETS_KEY_FILE");
        let data_dir = unique_target_dir("file-secrets-data");
        let config_dir = unique_target_dir("file-secrets-config");
        Self {
            store: FileSecretStore::open(data_dir.clone(), config_dir.clone(), None).unwrap(),
            key_path: config_dir.join("secrets.key"),
            store_path: data_dir.join("secrets.enc"),
            pending_path: data_dir.join("secrets.enc.pending"),
            data_dir,
            config_dir,
            _clear_key_env: clear_key_env,
        }
    }

    fn reopen(&self) -> FileSecretStore {
        FileSecretStore::open(self.data_dir.clone(), self.config_dir.clone(), None).unwrap()
    }

    fn store_backup_count(&self) -> usize {
        fs::read_dir(&self.data_dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with("secrets.old-") && name.ends_with(".enc"))
            .count()
    }
}

#[test]
fn uninitialized_then_raw_key_roundtrip() {
    let fixture = Fixture::new();
    assert_eq!(fixture.store.status().unwrap(), SecretStoreStatus::Uninitialized);
    assert_eq!(fixture.store.identifier_prefix(), SECRETSFILE_PREFIX);

    fixture.store.initialize_store("").unwrap();
    assert_eq!(fixture.store.status().unwrap(), SecretStoreStatus::Unlocked);
    assert!(fixture.key_path.is_file());
    assert!(fixture.store_path.is_file());
    let key_mode = fs::metadata(&fixture.key_path).unwrap().permissions().mode() & 0o777;
    assert_eq!(key_mode, 0o600);
    let key_raw = fs::read(&fixture.key_path).unwrap();
    assert!(!key_raw.trim_ascii().starts_with(b"{"));

    fixture.store.set("alpha", "one").unwrap();
    assert_eq!(fixture.store.get("alpha").unwrap().as_ref().map(|s| s.as_str()), Some("one"));
    fixture.store.delete("alpha").unwrap();
    assert_eq!(fixture.store.get("alpha").unwrap(), None);

    let reopened = fixture.reopen();
    assert_eq!(reopened.status().unwrap(), SecretStoreStatus::Unlocked);
    reopened.set("beta", "two").unwrap();
    assert_eq!(reopened.get("beta").unwrap().as_ref().map(|s| s.as_str()), Some("two"));
}

#[test]
fn passphrase_init_locks_until_unlock() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("correct-pass").unwrap();
    assert_eq!(fixture.store.status().unwrap(), SecretStoreStatus::UnlockedWithPassphrase);
    fixture.store.set("token", "secret-value").unwrap();

    let reopened = fixture.reopen();
    assert_eq!(reopened.status().unwrap(), SecretStoreStatus::Locked);
    assert!(reopened.get("token").unwrap_err().to_string().contains("locked"));
    assert!(reopened.set("token", "x").unwrap_err().to_string().contains("locked"));
    assert!(reopened.initialize_store("other").unwrap_err().to_string().contains("already initialized"));

    assert!(reopened.unlock_store("wrong-pass").unwrap_err().to_string().contains("Failed to unlock"));
    reopened.unlock_store("correct-pass").unwrap();
    assert_eq!(reopened.status().unwrap(), SecretStoreStatus::UnlockedWithPassphrase);
    assert_eq!(reopened.get("token").unwrap().as_ref().map(|s| s.as_str()), Some("secret-value"));
}

#[test]
fn protect_change_and_unprotect() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("").unwrap();
    fixture.store.set("keep", "alive").unwrap();

    fixture.store.protect_store("first-pass").unwrap();
    assert_eq!(fixture.store.status().unwrap(), SecretStoreStatus::UnlockedWithPassphrase);
    assert!(fs::read(&fixture.key_path).unwrap().trim_ascii().starts_with(b"{"));
    assert_eq!(fixture.store.get("keep").unwrap().as_ref().map(|s| s.as_str()), Some("alive"));

    fixture.store.change_store_passphrase("first-pass", "second-pass").unwrap();
    let reopened = fixture.reopen();
    assert!(reopened.unlock_store("first-pass").is_err());
    reopened.unlock_store("second-pass").unwrap();
    assert_eq!(reopened.get("keep").unwrap().as_ref().map(|s| s.as_str()), Some("alive"));

    reopened.unprotect_store("second-pass").unwrap();
    assert_eq!(reopened.status().unwrap(), SecretStoreStatus::Unlocked);
    let raw_again = fixture.reopen();
    assert_eq!(raw_again.status().unwrap(), SecretStoreStatus::Unlocked);
    assert_eq!(raw_again.get("keep").unwrap().as_ref().map(|s| s.as_str()), Some("alive"));
}

#[test]
fn set_store_passphrase_routes_by_status() {
    let fixture = Fixture::new();
    set_store_passphrase(&fixture.store, "", "routed-pass").unwrap();
    assert_eq!(fixture.store.status().unwrap(), SecretStoreStatus::UnlockedWithPassphrase);

    let locked = fixture.reopen();
    assert!(set_store_passphrase(&locked, "routed-pass", "next").unwrap_err().to_string().contains("locked"));
    locked.unlock_store("routed-pass").unwrap();
    set_store_passphrase(&locked, "routed-pass", "next-pass").unwrap();
    set_store_passphrase(&locked, "next-pass", "").unwrap();
    assert_eq!(locked.status().unwrap(), SecretStoreStatus::Unlocked);
}

#[test]
fn reinitialize_backs_up_orphaned_store() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("").unwrap();
    fixture.store.set("orphan", "value").unwrap();
    fs::remove_file(&fixture.key_path).unwrap();

    let orphaned = fixture.reopen();
    assert_eq!(orphaned.status().unwrap(), SecretStoreStatus::NoKey);
    orphaned.initialize_store("").unwrap();
    assert_eq!(orphaned.status().unwrap(), SecretStoreStatus::Unlocked);
    assert_eq!(orphaned.get("orphan").unwrap(), None);
    assert_eq!(fixture.store_backup_count(), 1);
}

#[test]
fn explicit_key_path_does_not_autogenerate() {
    let _clear_key_env = EnvVarGuard::remove("LIGHTKEEPER_SECRETS_KEY_FILE");
    let data_dir = unique_target_dir("file-secrets-explicit-data");
    let config_dir = unique_target_dir("file-secrets-explicit-config");
    let key_path = unique_target_dir("file-secrets-explicit-key").join("custom.key");

    let store = FileSecretStore::open(data_dir, config_dir, Some(key_path.clone())).unwrap();
    assert_eq!(store.status().unwrap(), SecretStoreStatus::NoKey);
    let error = store.initialize_store("").unwrap_err().to_string();
    assert!(error.contains(key_path.to_string_lossy().as_ref()));
    assert!(!key_path.exists());
}

#[test]
fn short_passphrase_rejected_before_backup() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("").unwrap();
    fixture.store.set("keep", "me").unwrap();
    fs::remove_file(&fixture.key_path).unwrap();
    let store_bytes = fs::read(&fixture.store_path).unwrap();

    let orphaned = fixture.reopen();
    assert!(orphaned.initialize_store("abc").unwrap_err().to_string().contains("Passphrase must be"));
    assert_eq!(fs::read(&fixture.store_path).unwrap(), store_bytes);
    assert_eq!(fixture.store_backup_count(), 0);
}

#[test]
fn over_max_kdf_params_fail_unlock() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("passphrase").unwrap();
    // 16-byte and 24-byte zero buffers, base64-encoded.
    let hostile = r#"{
        "format": "lightkeeper-secrets-key",
        "version": 1,
        "kdf": "argon2id",
        "m_cost": 2097152,
        "t_cost": 2,
        "p_cost": 1,
        "salt": "AAAAAAAAAAAAAAAAAAAAAA==",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "wrapped_key": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    }"#;
    fs::write(&fixture.key_path, hostile).unwrap();

    let locked = fixture.reopen();
    assert_eq!(locked.status().unwrap(), SecretStoreStatus::Locked);
    assert!(locked.unlock_store("passphrase").unwrap_err().to_string().contains("Failed to unlock"));
}

#[test]
fn tampered_protected_key_fails_unlock() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("passphrase").unwrap();
    let original = String::from_utf8(fs::read(&fixture.key_path).unwrap()).unwrap();
    let salt_key = "\"salt\": \"";
    let salt_start = original.find(salt_key).unwrap() + salt_key.len();
    let salt_end = original[salt_start..].find('"').unwrap() + salt_start;
    let mut tampered = original;
    tampered.replace_range(salt_start..salt_end, "AQEBAQEBAQEBAQEBAQEBAQ==");
    fs::write(&fixture.key_path, tampered).unwrap();

    let locked = fixture.reopen();
    assert!(locked.unlock_store("passphrase").unwrap_err().to_string().contains("Failed to unlock"));
}

#[test]
fn interrupted_protect_recovers_pending_store() {
    let fixture = Fixture::new();
    fixture.store.initialize_store("").unwrap();
    fixture.store.set("recover-me", "yes").unwrap();
    fixture.store.protect_store("passphrase").unwrap();

    let good_store = fs::read(&fixture.store_path).unwrap();
    fs::write(&fixture.store_path, b"not-valid-store").unwrap();
    fs::write(&fixture.pending_path, &good_store).unwrap();

    let reopened = fixture.reopen();
    reopened.unlock_store("passphrase").unwrap();
    assert_eq!(reopened.get("recover-me").unwrap().as_ref().map(|s| s.as_str()), Some("yes"));
    assert!(!fixture.pending_path.exists());
}