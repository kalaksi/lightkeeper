/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use lightkeeper::backend::{build_desktop_rsync_command, CommandBackend, RemoteCommandBackend, RemoteCoreClient};
use lightkeeper::configuration::CoreConnectionProfile;
use lightkeeper::module::command::internal::filebrowser::download::RsyncSsh;
use lightkeeper::remote_core::protocol::{FileTransferDirection, FileTransferPlan};
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::remote_core::server::run_remote_client_session;

use crate::remote_core::commands::{stub_ssh_factory, temp_config_dir_for_remote_core};
use crate::MemorySecretStore;

fn upload_plan() -> FileTransferPlan {
    FileTransferPlan {
        invocation_id: 1,
        direction: FileTransferDirection::Upload,
        address: String::from("host"),
        remote_path: String::from("/tmp"),
        ssh: RsyncSsh {
            username: String::from("user"),
            port: 22,
            private_key_path: Some(String::from("/core/key")),
            verify_host_key: true,
            custom_known_hosts_path: Some(PathBuf::from("/core/known_hosts")),
        },
        use_sudo_rsync: false,
    }
}

#[test]
fn rsync_remote_spec_trailing_slash_by_direction() {
    let ssh = RsyncSsh::new(&HashMap::from([(String::from("username"), String::from("alice"))]));
    assert_eq!(ssh.remote_spec("host.example", "/var/tmp", true).unwrap(), "alice@host.example:/var/tmp/");
    assert_eq!(ssh.remote_spec("host.example", "/var/tmp/", false).unwrap(), "alice@host.example:/var/tmp");
    assert_eq!(ssh.remote_spec("host.example", "/", false).unwrap(), "alice@host.example:/");

    let ssh = RsyncSsh::new(&HashMap::new());
    assert_eq!(ssh.remote_spec("10.0.0.1", "/var/tmp/", true).unwrap(), "10.0.0.1:/var/tmp/");
}

#[test]
fn desktop_rsync_rejects_malicious_plan() {
    let _xdg_config = crate::redirect_xdg_config_home();
    let profile = CoreConnectionProfile::default();
    for address in ["-oProxyCommand=touch /tmp/pwned", "/home/user/.bashrc", "host:", "a b", ""] {
        let plan = FileTransferPlan { address: address.to_string(), ..upload_plan() };
        assert!(build_desktop_rsync_command(&profile, &plan, "/local/file").is_err());
    }

    let mut plan = upload_plan();
    plan.ssh.username = String::from("--rsh=touch /tmp/pwned");
    assert!(build_desktop_rsync_command(&profile, &plan, "/local/file").is_err());

    let mut plan = upload_plan();
    plan.ssh.private_key_path = Some(String::from(r#"x" "-oProxyCommand=touch /tmp/pwned"#));
    assert!(build_desktop_rsync_command(&profile, &plan, "/local/file").is_err());
}

#[test]
fn desktop_rsync_protects_remote_file_names() {
    let _xdg_config = crate::redirect_xdg_config_home();
    let command = build_desktop_rsync_command(&CoreConnectionProfile::default(), &upload_plan(), "/local/file")
        .unwrap()
        .to_vec();
    assert!(command.contains(&String::from("--protect-args")));
}

#[test]
fn desktop_rsync_local_core_uses_desktop_known_hosts() {
    let _xdg_config = crate::redirect_xdg_config_home();
    let desktop_known_hosts = lightkeeper::file_handler::get_config_dir().join("known_hosts");
    let command = build_desktop_rsync_command(&CoreConnectionProfile::default(), &upload_plan(), "/local/file")
        .unwrap()
        .to_vec();
    let rsh = &command[command.iter().position(|argument| argument == "-e").unwrap() + 1];

    assert!(rsh.contains(r#""-i" "/core/key""#));
    assert!(rsh.contains(&format!("UserKnownHostsFile={}", desktop_known_hosts.display())));
    assert!(!rsh.contains("/core/known_hosts"), "local core must use desktop known_hosts, not core paths");
}

#[test]
fn desktop_rsync_nests_inner_ssh_through_core() {
    let _xdg_config = crate::redirect_xdg_config_home();
    let profile = CoreConnectionProfile {
        host: String::from("core.example"),
        port: Some(2222),
        username: Some(String::from("coreuser")),
        private_key_path: Some(String::from("/home/user/.ssh/id_ed25519")),
        ..Default::default()
    };
    let command = build_desktop_rsync_command(&profile, &upload_plan(), "/local/file").unwrap().to_vec();
    let rsh = &command[command.iter().position(|argument| argument == "-e").unwrap() + 1];

    assert!(rsh.starts_with(r#""ssh" "-o" "BatchMode=yes" "-o" "UserKnownHostsFile="#));
    assert!(!rsh[..rsh.find("coreuser@core.example").unwrap()].contains("/core/known_hosts"));
    assert!(rsh.ends_with(concat!(
        r#""-p" "2222" "-i" "/home/user/.ssh/id_ed25519" "coreuser@core.example" "#,
        r#""'ssh' '-p' '22' '-i' '/core/key' '-o' 'UserKnownHostsFile=/core/known_hosts' "#,
        r#"'-o' 'GlobalKnownHostsFile=/dev/null'""#,
    )));
}

#[test]
fn desktop_rsync_orders_paths_by_direction() {
    let _xdg_config = crate::redirect_xdg_config_home();
    let profile = CoreConnectionProfile::default();
    let command = build_desktop_rsync_command(&profile, &upload_plan(), "/local/file").unwrap().to_vec();
    assert_eq!(command[command.len() - 2..], [String::from("/local/file"), String::from("user@host:/tmp/")]);

    let download = FileTransferPlan {
        direction: FileTransferDirection::Download,
        remote_path: String::from("/tmp/file"),
        ..upload_plan()
    };
    let command = build_desktop_rsync_command(&profile, &download, "/local/dir").unwrap().to_vec();
    assert_eq!(command[command.len() - 2..], [String::from("user@host:/tmp/file"), String::from("/local/dir")]);
}

#[test]
fn remote_core_builds_file_transfer_plan() {
    let _ = env_logger::Builder::from_default_env().is_test(true).try_init();
    let _xdg_cache = crate::redirect_xdg_cache_home();
    let _xdg_config = crate::redirect_xdg_config_home();
    let (config_dir, main_config, hosts) = temp_config_dir_for_remote_core();
    let factory = Arc::new(stub_ssh_factory());
    let secret_store = Arc::new(MemorySecretStore::new());
    let mut runtime = CoreRuntime::new_with(&main_config, &hosts, factory, config_dir, secret_store).unwrap();

    let (client_stream, server) = UnixStream::pair().unwrap();
    let session_active = Arc::new(Mutex::new(false));

    let client_thread = thread::spawn(move || {
        let (ui_tx, ui_rx) = mpsc::channel();
        let core_client = Arc::new(RemoteCoreClient::new(PathBuf::from("_unused")));
        let mut backend = RemoteCommandBackend::new(core_client.clone());
        backend.connect_with_frontend_stream(ui_tx, client_stream).unwrap();
        let _ = ui_rx.recv_timeout(Duration::from_secs(5)).unwrap();

        let upload = core_client
            .build_file_transfer_plan("test-host", "_internal-filebrowser-upload", "/tmp/dest")
            .unwrap();
        assert!(upload.invocation_id > 0);
        assert_eq!(upload.direction, FileTransferDirection::Upload);
        assert_eq!(upload.address, "127.0.0.1");
        assert_eq!(upload.remote_path, "/tmp/dest");
        assert!(upload.ssh.custom_known_hosts_path.is_some());
        assert!(upload.use_sudo_rsync);

        let download = core_client
            .build_file_transfer_plan("test-host", "_internal-filebrowser-download", "/tmp/src/")
            .unwrap();
        assert_eq!(download.direction, FileTransferDirection::Download);
        assert_eq!(download.remote_path, "/tmp/src/");
        assert_ne!(download.invocation_id, upload.invocation_id);

        assert!(core_client.build_file_transfer_plan("test-host", "_internal-filebrowser-rm", "/tmp").is_err());

        backend.stop();
    });

    run_remote_client_session(server, &mut runtime, &session_active).unwrap();
    client_thread.join().unwrap();
}
