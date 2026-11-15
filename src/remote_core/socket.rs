/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::fs;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
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

/// Rejects peers whose kernel-attested UID differs from this process (Linux `SO_PEERCRED`).
#[cfg(target_os = "linux")]
pub fn verify_peer_uid(stream: &UnixStream) -> io::Result<()> {
    let mut cred = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut length = std::mem::size_of_val(&cred) as libc::socklen_t;
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            &mut cred as *mut _ as *mut libc::c_void,
            &mut length,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }

    let our_uid = file_handler::effective_uid()?;
    if cred.uid != our_uid {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "peer uid {} does not match effective uid {}",
                cred.uid, our_uid
            ),
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn verify_peer_uid(_stream: &UnixStream) -> io::Result<()> {
    Ok(())
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
