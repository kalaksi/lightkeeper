/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fmt::Display;

/// Runtime connection lifecycle for the desktop to admin-host core link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoreConnectionState {
    #[default]
    Disconnected,
    ConnectingSsh,
    Handshaking,
    Connected,
    Reconnecting,
    Failed,
}

impl Display for CoreConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreConnectionState::Disconnected => write!(f, "disconnected"),
            CoreConnectionState::ConnectingSsh => write!(f, "connecting_ssh"),
            CoreConnectionState::Handshaking => write!(f, "handshaking"),
            CoreConnectionState::Connected => write!(f, "connected"),
            CoreConnectionState::Reconnecting => write!(f, "reconnecting"),
            CoreConnectionState::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CoreConnectionStatus {
    pub state: CoreConnectionState,
    /// Last actionable transport or handshake error for the UI.
    pub last_error: Option<String>,
}

impl CoreConnectionStatus {
    pub fn set_state(&mut self, state: CoreConnectionState) {
        self.state = state;
        match state {
            CoreConnectionState::Connected | CoreConnectionState::Disconnected => {
                self.last_error = None;
            }
            CoreConnectionState::Failed => {}
            _ => {}
        }
    }

    pub fn set_failed(&mut self, message: impl Into<String>) {
        self.state = CoreConnectionState::Failed;
        self.last_error = Some(message.into());
    }
}
