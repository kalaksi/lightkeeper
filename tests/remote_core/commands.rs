/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::{BTreeMap, HashMap};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use lightkeeper::backend::{CommandBackend, ConfigBackend, RemoteCommandBackend, RemoteConfigBackend, RemoteCoreClient};
use lightkeeper::configuration::{self, get_default_main_config, Configuration, Groups};
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
    // Use `overrides`, not `effective`: `write_hosts_config` clears `effective` before serializing.
    host_settings.overrides.host_settings = vec![HostSetting::UseSudo];
    host_settings.overrides.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default(),
    );
    host_settings.overrides.commands.insert(
        systemd::service::Start::get_metadata().module_spec.id.clone(),
        configuration::CommandConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.overrides.monitors.insert(
        Os::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.overrides.monitors.insert(
        Service::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        },
    );
    host_settings.overrides.custom_commands.push(configuration::CustomCommandConfig {
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

fn temp_config_dir_for_remote_core() -> (String, configuration::Configuration, configuration::Hosts) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("lk-remote-core-test-{}-{}", std::process::id(), nanos));

    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let dir_str = dir.to_string_lossy().to_string();
    Configuration::write_initial_config(&dir).unwrap();
    let groups: Groups = serde_yaml::from_str(lightkeeper::configuration::DEFAULT_GROUPS_CONFIG).unwrap();
    let hosts_stub = stub_hosts();
    let main_config = get_default_main_config();
    Configuration::write_main_config(&dir_str, &main_config).unwrap();
    Configuration::write_hosts_config(&dir_str, &hosts_stub).unwrap();
    Configuration::write_groups_config(&dir_str, &groups).unwrap();

    let (main_config, mut hosts, _groups) = Configuration::read(&dir_str).unwrap();
    hosts.predefined_platforms = hosts_stub.predefined_platforms.clone();

    (dir_str, main_config, hosts)
}

fn with_remote_core_session(
    client_body: impl FnOnce(RemoteCommandBackend, RemoteConfigBackend, mpsc::Receiver<UIUpdate>) + Send + 'static,
) {
    let (config_dir, main_config, hosts) = temp_config_dir_for_remote_core();
    let factory = Arc::new(stub_ssh_factory());
    let mut runtime = CoreRuntime::new_with_module_factory(&main_config, &hosts, factory, config_dir).unwrap();

    let (client_stream, server) = UnixStream::pair().unwrap();
    let session_active = Arc::new(Mutex::new(false));

    let client_thread = thread::spawn(move || {
        let (ui_tx, ui_rx) = mpsc::channel();
        let core_client = Arc::new(RemoteCoreClient::new(PathBuf::from("_unused")));
        let mut backend = RemoteCommandBackend::new(core_client.clone());
        backend.connect_with_frontend_stream(ui_tx, client_stream).unwrap();

        let _ = ui_rx.recv_timeout(Duration::from_secs(5)).unwrap();

        let config_backend = RemoteConfigBackend::new(core_client);
        client_body(backend, config_backend, ui_rx);
    });

    run_remote_client_session(server, &mut runtime, &session_active).unwrap();
    client_thread.join().unwrap();
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

        let update = ui_rx.recv_timeout(remaining.min(Duration::from_millis(300))).unwrap();
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

    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();
    let custom_id = CustomCommand::get_metadata().module_spec.id.clone();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        let commands = backend.commands_for_host(TEST_HOST).unwrap();
        assert!(commands.contains_key(&systemd_start_id));
        assert!(commands.contains_key(&custom_id));

        let command = backend.command_for_host(TEST_HOST, &systemd_start_id).unwrap().unwrap();
        assert_eq!(command.command_id, systemd_start_id);
        assert!(backend.command_for_host(TEST_HOST, "no-such-command").unwrap().is_none());

        let custom = backend.custom_commands_for_host(TEST_HOST).unwrap();
        let entry = custom.get("custom-1").unwrap();
        assert_eq!(entry.command, "echo test-service");

        backend.stop();
    });
}

#[test]
fn remote_core_execute_command() {
    init_log();

    let custom_id = CustomCommand::get_metadata().module_spec.id.clone();

    with_remote_core_session(move |mut backend, _cfg, ui_rx| {
        let invocation_id = backend
            .execute_command(
                TEST_HOST,
                &custom_id,
                &["echo test-service".to_string()],
            )
            .unwrap();
        assert!(invocation_id > 0, "expected invocation id");

        let display = recv_host_until(&ui_rx, TEST_HOST, |d| {
            d.host_state
                .command_results
                .get(&custom_id)
                .is_some_and(|result| {
                    result.progress == 100 && result.message.contains("stub-line-from-fake-ssh")
                })
        });

        let result = display.host_state.command_results.get(&custom_id).unwrap();
        assert!(result.message.contains("stub-line-from-fake-ssh"));

        backend.stop();
    });
}

#[test]
fn remote_core_all_host_categories() {
    init_log();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        let mut categories = backend.all_host_categories(TEST_HOST).unwrap();
        categories.sort();
        assert_eq!(categories, vec!["host".to_string(), "systemd".to_string()]);

        backend.stop();
    });
}

#[test]
fn remote_core_initialize_hosts() {
    init_log();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        let mut host_ids = backend.initialize_hosts().unwrap();
        host_ids.sort();
        assert!(host_ids.contains(&TEST_HOST.to_string()));

        backend.stop();
    });
}

#[test]
fn remote_core_refresh_invocation_ids() {
    init_log();

    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        let for_command = backend.refresh_monitors_for_command(TEST_HOST, &systemd_start_id).unwrap();
        assert!(!for_command.is_empty());

        let for_category = backend.refresh_monitors_of_category(TEST_HOST, "host").unwrap();
        assert!(!for_category.is_empty());

        let _for_certs = backend.refresh_certificate_monitors().unwrap();

        backend.stop();
    });
}

#[test]
fn remote_core_resolve_text_editor_path() {
    init_log();

    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        let path = backend
            .resolve_text_editor_path(
                TEST_HOST,
                &systemd_start_id,
                &["/tmp/editor-target".to_string()],
            )
            .unwrap();
        assert_eq!(path.as_deref(), Some("/tmp/editor-target"));

        backend.stop();
    });
}

#[test]
fn remote_core_download_editable_file() {
    init_log();

    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        let (invocation_id, path) = backend
            .download_editable_file(TEST_HOST, &systemd_start_id, "test-service")
            .unwrap();
        assert!(invocation_id > 0);
        assert!(!path.is_empty());

        backend.stop();
    });
}

#[test]
fn remote_core_write_cache_and_upload() {
    init_log();

    let systemd_start_id = systemd::service::Start::get_metadata().module_spec.id.clone();

    with_remote_core_session(move |mut backend, _cfg, ui_rx| {
        backend.download_editable_file(TEST_HOST, &systemd_start_id, "test-service").unwrap();

        recv_host_until(&ui_rx, TEST_HOST, |d| {
            d.host_state
                .command_results
                .get(&systemd_start_id)
                .is_some_and(|result| result.progress == 100)
        });

        backend.write_cached_file(TEST_HOST, "test-service", b"new-bytes".to_vec()).unwrap();
        let invocation_id = backend.upload_file_from_cache(TEST_HOST, &systemd_start_id, "test-service").unwrap();
        assert!(invocation_id > 0);

        backend.stop();
    });
}

#[test]
fn remote_core_send_nowait_messages() {
    init_log();

    with_remote_core_session(move |mut backend, _cfg, _ui_rx| {
        backend.refresh_host_monitors(TEST_HOST);
        backend.verify_host_key(TEST_HOST, "ssh", "test-key");
        backend.initialize_host(TEST_HOST);
        backend.interrupt_invocation(99);

        backend.stop();
    });
}

#[test]
fn remote_core_get_update_config() {
    init_log();

    with_remote_core_session(move |mut backend, config, ui_rx| {
        let _ = ui_rx.recv_timeout(Duration::from_secs(5));
        let (mut main0, hosts0, groups0) = config.get_config().unwrap();
        assert!(!main0.preferences.show_charts);

        main0.preferences.show_charts = true;
        config.update_config(main0, hosts0, groups0).unwrap();
        let (main2, _, _) = config.get_config().unwrap();
        assert!(main2.preferences.show_charts);

        backend.stop();
    });
}
