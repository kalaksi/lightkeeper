/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::fmt::Display;

use crate::secrets_manager::SecretStoreStatus;

/// Runtime connection lifecycle for the desktop to remote core host link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoreConnectionState {
    #[default]
    Disconnected,
    ConnectingSsh,
    Handshaking,
    Connected,
    Failed,
}

impl Display for CoreConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreConnectionState::Disconnected => write!(f, "disconnected"),
            CoreConnectionState::ConnectingSsh => write!(f, "connecting_ssh"),
            CoreConnectionState::Handshaking => write!(f, "handshaking"),
            CoreConnectionState::Connected => write!(f, "connected"),
            CoreConnectionState::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CoreConnectionStatus {
    pub state: CoreConnectionState,
    /// Last actionable transport or handshake error for the UI.
    pub last_error: Option<String>,
    /// Desktop and core protocol versions differ, core needs an upgrade (or downgrade).
    pub version_mismatch: bool,
    /// Received in handshake and secret store action results.
    pub secret_store_status: Option<SecretStoreStatus>,
}

impl CoreConnectionStatus {
    pub fn set_state(&mut self, state: CoreConnectionState) {
        self.state = state;
        match state {
            CoreConnectionState::Connected => {
                self.last_error = None;
                self.version_mismatch = false;
            }
            CoreConnectionState::Disconnected => {
                self.last_error = None;
                self.version_mismatch = false;
                self.secret_store_status = None;
            }
            CoreConnectionState::Failed => {}
            _ => {}
        }
    }

    pub fn set_failed(&mut self, message: impl Into<String>) {
        self.state = CoreConnectionState::Failed;
        self.last_error = Some(message.into());
        self.version_mismatch = false;
        self.secret_store_status = None;
    }
}
