/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

#![allow(clippy::redundant_field_names)]
#![allow(clippy::needless_return)]
#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::Parser;
use lightkeeper::Configuration;
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::remote_core::server::CoreServer;

#[derive(Parser, Clone)]
pub struct Args {
    #[clap(short, long, default_value = "")]
    pub config_dir: String,
    #[clap(long, default_value = "")]
    pub socket_path: String,
}

fn main() {
    std::env::set_var("LANGUAGE", "en_US");

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    log::info!("Lightkeeper Core starting...");

    let (main_config, hosts_config, _group_config) = match Configuration::read(&args.config_dir) {
        Ok(configuration) => configuration,
        Err(error) => {
            log::error!("Error while reading configuration files: {}", error);
            return;
        }
    };

    let socket_path = if args.socket_path.is_empty() {
        match CoreRuntime::default_socket_path() {
            Ok(path) => path,
            Err(error) => {
                log::error!("Failed to determine socket path: {}", error);
                return;
            }
        }
    }
    else {
        PathBuf::from(args.socket_path.clone())
    };

    let runtime = match CoreRuntime::new(&main_config, &hosts_config) {
        Ok(runtime) => runtime,
        Err(error) => {
            log::error!("Failed to start backend runtime: {}", error);
            return;
        }
    };

    if let Err(error) = CoreServer::start(socket_path, runtime) {
        log::error!("Core server stopped: {}", error);
    }
}
