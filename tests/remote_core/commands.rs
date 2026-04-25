/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::{BTreeMap, HashMap};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lightkeeper::backend::{CommandBackend, RemoteCommandBackend};
use lightkeeper::configuration::{self, get_default_main_config};
use lightkeeper::frontend::{HostDisplayData, UIUpdate};
use lightkeeper::module::command::internal::custom_command::CustomCommand;
use lightkeeper::module::command::systemd;
use lightkeeper::module::command::CommandModule;
use lightkeeper::module::connection::Connector;
use lightkeeper::module::monitoring::os::Os;
use lightkeeper::module::monitoring::systemd::service::Service;
use lightkeeper::module::monitoring::MonitoringModule;
use lightkeeper::module::platform_info::{Flavor, PlatformInfo};
use lightkeeper::module::MetadataSupport;
use lightkeeper::remote_core::runtime::CoreRuntime;
use lightkeeper::remote_core::server::run_remote_client_session;
use lightkeeper::HostSetting;
use lightkeeper::ModuleFactory;

use crate::{StubSsh2, StubTcp};

const TEST_HOST: &str = "test-host";

fn init_log() {
    let _ = env_logger::Builder::from_default_env().is_test(true).try_init();
}

fn stub_ssh_factory() -> ModuleFactory {
    const STUB_SSH_FALLBACK: &str = "stub-line-from-fake-ssh-connector-output\n";
    const BUSCTL_LIST_UNITS: &str = concat!(
        r#""busctl" "--no-pager" "--json=short" "call" "org.freedesktop.systemd1" "#,
        r#""/org/freedesktop/systemd1" "org.freedesktop.systemd1.Manager" "ListUnits""#
    );
    const BUSCTL_JSON: &str = r#"{
  "type": "a(ssssssouso)",
  "data": [
    [
      ["ssh.service", "OpenSSH server", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/ssh_2eservice", 0, "", "/"],
      ["test-service.service", "test", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/test_2eservice", 0, "", "/"]
    ]
  ]
}"#;

    let new_stub_ssh = move |_settings: &HashMap<String, String>| -> Connector {
        let mut ssh = StubSsh2::default();
        ssh.add_response(BUSCTL_LIST_UNITS, BUSCTL_JSON, 0);
        ssh.add_response("_", STUB_SSH_FALLBACK, 0);
        Box::new(ssh) as Connector
    };

    ModuleFactory::new_with(
        vec![
            (StubTcp::get_metadata(), |_settings: &HashMap<String, String>| StubTcp::new_any("")),
            (StubSsh2::get_metadata(), new_stub_ssh),
        ],
        vec![
            (Os::get_metadata(), Os::new_monitoring_module),
            (Service::get_metadata(), Service::new_monitoring_module),
        ],
        vec![(
            systemd::service::Start::get_metadata(),
            systemd::service::Start::new_command_module,
        )],
    )
}

fn stub_hosts() -> configuration::Hosts {
    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    host_settings.effective.host_settings = vec![HostSetting::UseSudo];
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default(),
    );
    host_settings.effective.commands.insert(
        systemd::service::Start::get_metadata().module_spec.id.clone(),
        configuration::CommandConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.effective.monitors.insert(
        Os::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.effective.monitors.insert(
        Service::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.effective.custom_commands.push(configuration::CustomCommandConfig {
        name: "custom-1".to_string(),
        description: String::new(),
        command: "echo test-service".to_string(),
    });

    configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::from([(
            TEST_HOST.to_string(),
            PlatformInfo::linux(Flavor::Debian, "12.0"),
        )]),
        ..Default::default()
    }
}

fn with_remote_core_session(
    hosts: &configuration::Hosts,
    client_body: impl FnOnce(RemoteCommandBackend, mpsc::Receiver<UIUpdate>) + Send + 'static,
) {
    let main_config = get_default_main_config();
    let factory = Arc::new(stub_ssh_factory());
    let mut runtime = CoreRuntime::new_with_module_factory(&main_config, hosts, factory).expect("CoreRuntime");

    let (client_stream, server) = UnixStream::pair().expect("pair");
    let session_active = Arc::new(Mutex::new(false));

    let client_thread = thread::spawn(move || {
        let (ui_tx, ui_rx) = mpsc::channel();
        let mut backend = RemoteCommandBackend::new(PathBuf::from("_unused"));
        backend
            .connect_with_frontend_stream(ui_tx, client_stream)
            .expect("connect");

        let _ = ui_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("initial host UI update");

        client_body(backend, ui_rx);
    });

    run_remote_client_session(server, &mut runtime, &session_active).expect("core session");
    client_thread.join().expect("client thread");
}

fn recv_host_until(
    ui_rx: &mpsc::Receiver<UIUpdate>,
    host_name: &str,
    mut pred: impl FnMut(&HostDisplayData) -> bool,
) -> HostDisplayData {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            panic!("timeout waiting for host UI update");
        }
        let update = ui_rx
            .recv_timeout(remaining.min(Duration::from_millis(300)))
            .expect("ui channel closed");
        if let UIUpdate::Host(display) = update {
            if display.host_state.host.name == host_name && pred(&display) {
                return display;
            }
        }
    }
}

#[test]
fn remote_core_commands_for_host() {
    init_log();

    let hosts = stub_hosts();
    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();
    let custom_id = CustomCommand::get_metadata().module_spec.id.clone();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        let commands = backend.commands_for_host(TEST_HOST).expect("commands_for_host");
        assert!(commands.contains_key(&systemd_start_id));
        assert!(commands.contains_key(&custom_id));

        let command = backend
            .command_for_host(TEST_HOST, &systemd_start_id)
            .expect("command_for_host")
            .expect("start command");
        assert_eq!(command.command_id, systemd_start_id);

        assert!(backend
            .command_for_host(TEST_HOST, "no-such-command")
            .expect("command_for_host")
            .is_none());

        let custom = backend.custom_commands_for_host(TEST_HOST).expect("custom_commands_for_host");
        let entry = custom.get("custom-1").expect("custom-1");
        assert_eq!(entry.command, "echo test-service");

        backend.stop();
    });
}

#[test]
fn remote_core_execute_command() {
    init_log();

    let hosts = stub_hosts();
    let custom_id = CustomCommand::get_metadata().module_spec.id.clone();

    with_remote_core_session(&hosts, move |mut backend, ui_rx| {
        let invocation_id = backend
            .execute_command(
                TEST_HOST,
                &custom_id,
                &["echo test-service".to_string()],
            )
            .expect("execute_command");
        assert!(invocation_id > 0, "expected invocation id");

        let display = recv_host_until(&ui_rx, TEST_HOST, |d| {
            d.host_state
                .command_results
                .get(&custom_id)
                .is_some_and(|result| {
                    result.progress == 100 && result.message.contains("stub-line-from-fake-ssh")
                })
        });

        let result = display.host_state.command_results.get(&custom_id).expect("command result");
        assert!(result.message.contains("stub-line-from-fake-ssh"));

        backend.stop();
    });
}

#[test]
fn remote_core_all_host_categories() {
    init_log();

    let hosts = stub_hosts();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        let mut categories = backend.all_host_categories(TEST_HOST).expect("all_host_categories");
        categories.sort();
        assert_eq!(categories, vec!["host".to_string(), "systemd".to_string()]);

        backend.stop();
    });
}

#[test]
fn remote_core_initialize_hosts() {
    init_log();

    let hosts = stub_hosts();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        let mut host_ids = backend.initialize_hosts().expect("initialize_hosts");
        host_ids.sort();
        assert!(host_ids.contains(&TEST_HOST.to_string()));

        backend.stop();
    });
}

#[test]
fn remote_core_refresh_invocation_ids() {
    init_log();

    let hosts = stub_hosts();
    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        let for_command = backend
            .refresh_monitors_for_command(TEST_HOST, &systemd_start_id)
            .expect("refresh_monitors_for_command");
        assert!(!for_command.is_empty());

        let for_category = backend
            .refresh_monitors_of_category(TEST_HOST, "host")
            .expect("refresh_monitors_of_category");
        assert!(!for_category.is_empty());

        let _for_certs = backend
            .refresh_certificate_monitors()
            .expect("refresh_certificate_monitors");

        backend.stop();
    });
}

#[test]
fn remote_core_resolve_text_editor_path() {
    init_log();

    let hosts = stub_hosts();
    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        let path = backend
            .resolve_text_editor_path(
                TEST_HOST,
                &systemd_start_id,
                &["/tmp/editor-target".to_string()],
            )
            .expect("resolve_text_editor_path");
        assert_eq!(path.as_deref(), Some("/tmp/editor-target"));

        backend.stop();
    });
}

#[test]
fn remote_core_download_editable_file() {
    init_log();

    let hosts = stub_hosts();
    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        let (invocation_id, path) = backend
            .download_editable_file(TEST_HOST, &systemd_start_id, "test-service")
            .expect("download_editable_file");
        assert!(invocation_id > 0);
        assert!(!path.is_empty());

        backend.stop();
    });
}

#[test]
fn remote_core_write_cache_and_upload() {
    init_log();

    let hosts = stub_hosts();
    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(&hosts, move |mut backend, ui_rx| {
        backend
            .download_editable_file(TEST_HOST, &systemd_start_id, "test-service")
            .expect("download_editable_file");

        recv_host_until(&ui_rx, TEST_HOST, |d| {
            d.host_state
                .command_results
                .get(&systemd_start_id)
                .is_some_and(|result| result.progress == 100)
        });

        backend
            .write_cached_file(TEST_HOST, "test-service", b"new-bytes".to_vec())
            .expect("write_cached_file");

        let invocation_id = backend
            .upload_file_from_cache(TEST_HOST, &systemd_start_id, "test-service")
            .expect("upload_file_from_cache");
        assert!(invocation_id > 0);

        backend.stop();
    });
}

#[test]
fn remote_core_send_nowait_messages() {
    init_log();

    let hosts = stub_hosts();

    with_remote_core_session(&hosts, move |mut backend, _ui_rx| {
        backend.refresh_host_monitors(TEST_HOST);
        backend.verify_host_key(TEST_HOST, "ssh", "test-key");
        backend.initialize_host(TEST_HOST);
        backend.interrupt_invocation(99);

        backend.stop();
    });
}
