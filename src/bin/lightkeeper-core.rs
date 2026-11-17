/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

#![allow(clippy::redundant_field_names)]
#![allow(clippy::needless_return)]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::Parser;
use lightkeeper::file_handler;
use lightkeeper::Configuration;
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::remote_core::server::{CoreServer, ShutdownSignals};
use lightkeeper::utils;

#[derive(Parser, Clone)]
pub struct Args {
    #[clap(short, long, default_value = "")]
    pub config_dir: String,
    #[clap(long, default_value = "")]
    pub socket_path: String,
    /// Optional path to the secrets wrapping key (file backend only).
    /// Defaults to $LIGHTKEEPER_SECRETS_KEY_FILE, then `<config-dir>/secrets.key`. The key is base64 of 32 bytes.
    /// To keep the key encrypted at rest, use systemd `LoadCredentialEncrypted=` and point this to
    /// `$CREDENTIALS_DIRECTORY/<name>`.
    #[clap(long)]
    pub secrets_key_file: Option<PathBuf>,
}

fn main() {
    std::env::set_var("LANGUAGE", "en_US");

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    if !args.config_dir.is_empty() {
        file_handler::set_config_dir_override(PathBuf::from(&args.config_dir));
    }

    log::info!("Lightkeeper Core starting...");

    if let Err(error) = utils::disable_process_dumping() {
        log::warn!("Failed to disable process dumping: {}", error);
    }

    let shutdown_signals = match ShutdownSignals::block() {
        Ok(shutdown_signals) => shutdown_signals,
        Err(error) => {
            log::error!("Failed to set up signal handling: {}", error);
            std::process::exit(1);
        }
    };

    let (main_config, hosts_config, _group_config) = match Configuration::read(&args.config_dir) {
        Ok(configuration) => configuration,
        Err(error) => {
            log::error!("Error while reading configuration files: {}", error);
            std::process::exit(1);
        }
    };

    let socket_path = if args.socket_path.is_empty() {
        match CoreRuntime::default_socket_path() {
            Ok(path) => path,
            Err(error) => {
                log::error!("Failed to determine socket path: {}", error);
                std::process::exit(1);
            }
        }
    }
    else {
        PathBuf::from(args.socket_path.clone())
    };

    let secrets_key_file = args.secrets_key_file.filter(|path| !path.as_os_str().is_empty());
    let runtime = match CoreRuntime::new(
        &main_config,
        &hosts_config,
        args.config_dir.clone(),
        secrets_key_file,
    ) {
        Ok(runtime) => runtime,
        Err(error) => {
            log::error!("Failed to start backend runtime: {}", error);
            std::process::exit(1);
        }
    };

    if let Err(error) = CoreServer::start(socket_path, runtime, shutdown_signals) {
        log::error!("Core server stopped: {}", error);
        std::process::exit(1);
    }
}
