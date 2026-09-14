/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::fs;
use std::io;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::Path;

use crate::error::LkError;
use crate::file_handler;

pub const SOCKET_DIR_MODE: u32 = 0o700;
pub const SOCKET_FILE_MODE: u32 = 0o600;

pub fn prepare_socket_path(socket_path: &Path) -> Result<(), LkError> {
    if let Some(parent_dir) = socket_path.parent() {
        fs::create_dir_all(parent_dir)?;
        fs::set_permissions(parent_dir, fs::Permissions::from_mode(SOCKET_DIR_MODE))?;
        verify_dir_owned_by_us(parent_dir)?;
    }

    remove_stale_socket(socket_path)?;
    Ok(())
}

/// Binds a Unix listener so the socket is created owner-only (no bind-then-chmod race).
pub fn bind_listener(socket_path: &Path) -> io::Result<UnixListener> {
    // 0777 & !0177 = 0600 for the socket inode at creation time.
    let old_umask = unsafe { libc::umask(0o177) };
    let bind_result = UnixListener::bind(socket_path);
    unsafe { libc::umask(old_umask) };

    let listener = bind_result?;
    set_socket_permissions(socket_path)?;
    Ok(listener)
}

pub fn set_socket_permissions(socket_path: &Path) -> io::Result<()> {
    fs::set_permissions(socket_path, fs::Permissions::from_mode(SOCKET_FILE_MODE))
}

pub fn remove_stale_socket(socket_path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(socket_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    if !metadata.file_type().is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("refusing to remove stale path {}: not a unix socket", socket_path.display(),),
        ));
    }

    let our_uid = file_handler::effective_uid()?;
    if metadata.uid() != our_uid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "refusing to remove stale socket {}: owned by uid {}, expected {}",
                socket_path.display(),
                metadata.uid(),
                our_uid,
            ),
        ));
    }

    fs::remove_file(socket_path)
}

fn verify_dir_owned_by_us(path: &Path) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a directory", path.display()),
        ));
    }

    let our_uid = file_handler::effective_uid()?;
    if metadata.uid() != our_uid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "socket directory {} owned by uid {}, expected {}",
                path.display(),
                metadata.uid(),
                our_uid,
            ),
        ));
    }

    let mode = metadata.permissions().mode() & 0o777;
    if mode != SOCKET_DIR_MODE {
        log::warn!(
            "socket directory {} mode is {:o}, expected {:o}",
            path.display(),
            mode,
            SOCKET_DIR_MODE,
        );
        fs::set_permissions(path, fs::Permissions::from_mode(SOCKET_DIR_MODE))?;
    }

    Ok(())
}
