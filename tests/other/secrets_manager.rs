/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::secrets_manager::detect_secret_backend;

#[test]
fn detect_secret_backend_empty_is_keyring() {
    assert_eq!(detect_secret_backend(""), "keyring");
}

#[test]
fn detect_secret_backend_native_placeholder_is_keyring() {
    assert_eq!(detect_secret_backend("keyring:abc"), "keyring");
}

#[test]
fn detect_secret_backend_portal_placeholder_is_keyring() {
    assert_eq!(detect_secret_backend("pkeyring:abc"), "keyring");
}

#[test]
fn detect_secret_backend_plaintext_value() {
    assert_eq!(detect_secret_backend("hunter2"), "plaintext");
}
