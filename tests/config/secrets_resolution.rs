/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc};
use std::time::Duration;

use lightkeeper::command_handler::CommandHandler;
use lightkeeper::configuration::{
    ConfigGroup, Configuration, ConnectorConfig, HostSettings, Hosts, MonitorConfig,
};
use lightkeeper::connection_manager::ConnectionManager;
use lightkeeper::frontend::UIUpdate;
use lightkeeper::host_manager::HostManager;
use lightkeeper::monitor_manager::MonitorManager;
use lightkeeper::CoreComponents;
use lightkeeper::module::connection::{ConnectionModule, Connector};
use lightkeeper::module::monitoring::linux::Who;
use lightkeeper::module::monitoring::MonitoringModule;
use lightkeeper::module::*;
use lightkeeper::error::LkError;
use lightkeeper::secrets_manager::{
    secret_lookup_key, Secret, SecretStore, KEYRING_PREFIX, SECRETSFILE_PREFIX,
};
use lightkeeper::ModuleFactory;

use crate::{MemorySecretStore, StubSsh2, StubTcp};

const HOST_ID: &str = "secret-host";
const USERNAME: &str = "deploy";

fn stub_factory() -> ModuleFactory {
    ModuleFactory::new_with(
        vec![
            (StubTcp::get_metadata(), |_settings| StubTcp::new_any("")),
            (StubSsh2::get_metadata(), StubSsh2::new_connection_module),
        ],
        vec![(Who::get_metadata(), Who::new_monitoring_module)],
        vec![],
    )
}

fn hosts_with_ssh_settings(password: &str) -> Hosts {
    let mut connector = ConnectorConfig::default();
    connector.settings.insert("username".to_string(), USERNAME.to_string());
    connector.settings.insert("password".to_string(), password.to_string());

    let mut effective = ConfigGroup::default();
    effective.connectors.insert("ssh".to_string(), connector);
    effective.monitors.insert("who".to_string(), MonitorConfig::default());

    let mut host = HostSettings::default();
    host.address = "127.0.0.1".to_string();
    host.effective = effective;

    Hosts {
        hosts: BTreeMap::from([(HOST_ID.to_string(), host)]),
        predefined_platforms: BTreeMap::new(),
        certificate_monitors: Vec::new(),
    }
}

fn password_placeholder() -> String {
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    format!("{}{}", KEYRING_PREFIX, lookup_key)
}

struct FailingSecretStore;

impl SecretStore for FailingSecretStore {
    fn identifier_prefix(&self) -> &'static str {
        KEYRING_PREFIX
    }

    fn get(&self, _key: &str) -> Result<Option<Secret>, LkError> {
        Err(LkError::other("Secret store is locked"))
    }

    fn set(&self, _key: &str, _value: &str) -> Result<(), LkError> {
        Ok(())
    }

    fn delete(&self, _key: &str) -> Result<(), LkError> {
        Ok(())
    }
}

fn configure(hosts: &mut Hosts, store: Arc<dyn SecretStore>) -> (ConnectionManager, Vec<(String, LkError)>) {
    let (runtime_secrets, errors) = Configuration::prepare_host_secrets(hosts, store);
    let mut connection_manager = ConnectionManager::new(Arc::new(stub_factory()));
    connection_manager.configure(hosts, &runtime_secrets);
    (connection_manager, errors)
}

fn configure_ssh(hosts: &mut Hosts, store: Arc<dyn SecretStore>) -> HashMap<String, String> {
    let (connection_manager, errors) = configure(hosts, store);
    assert!(errors.is_empty());
    ssh_settings(&connection_manager)
}

fn ssh_connector(connection_manager: &ConnectionManager) -> Option<Connector> {
    connection_manager.get_connector(HOST_ID, &ModuleSpecification::connector("ssh", "0.0.1"))
}

fn ssh_settings(connection_manager: &ConnectionManager) -> HashMap<String, String> {
    ssh_connector(connection_manager).unwrap().get_settings().unwrap().clone()
}

fn assert_ssh_skipped_with_error(hosts: &mut Hosts, store: Arc<dyn SecretStore>, expected_reason: &str) {
    let (connection_manager, errors) = configure(hosts, store);
    assert!(ssh_connector(&connection_manager).is_none());
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, HOST_ID);
    let expected_start = format!("Host {}, connector ssh secret 'password' {}", HOST_ID, expected_reason);
    assert!(errors[0].1.message.starts_with(&expected_start), "{}", errors[0].1.message);
}

fn next_error(receiver: &mpsc::Receiver<UIUpdate>) -> (String, String) {
    loop {
        if let UIUpdate::Host(display_data) = receiver.recv_timeout(Duration::from_secs(5)).unwrap() {
            if let Some(error) = display_data.new_errors.first() {
                return (display_data.host_state.host.name.clone(), error.message.clone());
            }
        }
    }
}

#[test]
fn stored_secret_is_injected_into_connector() {
    let store = Arc::new(MemorySecretStore::new());
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    store.set(&lookup_key, "stored-password").unwrap();

    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    let settings = configure_ssh(&mut hosts, store);

    assert_eq!(settings.get("password").map(String::as_str), Some("stored-password"));
}

#[test]
fn plaintext_password_reaches_connector_unchanged() {
    let store = Arc::new(MemorySecretStore::new());
    let mut hosts = hosts_with_ssh_settings("plaintext-password");
    let settings = configure_ssh(&mut hosts, store);

    assert_eq!(settings.get("password").map(String::as_str), Some("plaintext-password"));
}

#[test]
fn missing_secret_skips_connector_with_error() {
    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    assert_ssh_skipped_with_error(&mut hosts, Arc::new(MemorySecretStore::new()), "not found");
}

#[test]
fn unreadable_secret_skips_connector_with_error() {
    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    assert_ssh_skipped_with_error(&mut hosts, Arc::new(FailingSecretStore), "could not be read");
}

#[test]
fn wrong_backend_placeholder_skips_connector_with_error() {
    let store = Arc::new(MemorySecretStore::new());
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    store.set(&lookup_key, "must-not-leak").unwrap();

    let mut hosts = hosts_with_ssh_settings(&format!("{}{}", SECRETSFILE_PREFIX, lookup_key));
    assert_ssh_skipped_with_error(&mut hosts, store, "uses an incompatible secrets backend");
}

#[test]
fn secret_errors_reach_current_and_later_observers() {
    let module_factory = Arc::new(stub_factory());
    let host_manager = Rc::new(RefCell::new(HostManager::new()));
    let invocation_id_counter = Arc::new(AtomicU64::new(0));
    let mut core = CoreComponents {
        module_factory: module_factory.clone(),
        host_manager: host_manager.clone(),
        connection_manager: ConnectionManager::new(module_factory.clone()),
        monitor_manager: MonitorManager::new(host_manager.clone(), module_factory.clone(), invocation_id_counter.clone()),
        command_handler: CommandHandler::new(host_manager.clone(), module_factory, invocation_id_counter),
    };

    let (current_sender, current_receiver) = mpsc::channel();
    host_manager.borrow_mut().add_observer(current_sender);
    let hosts = hosts_with_ssh_settings(&password_placeholder());
    core.reconfigure(&Configuration::default(), &hosts, Arc::new(MemorySecretStore::new()), true);

    let (later_sender, later_receiver) = mpsc::channel();
    host_manager.borrow_mut().add_observer(later_sender.clone());
    host_manager.borrow().send_config_errors(&later_sender);

    let expected = (HOST_ID.to_string(), format!("Host {}, connector ssh secret 'password' not found", HOST_ID));
    assert_eq!(next_error(&current_receiver), expected);
    assert_eq!(next_error(&later_receiver), expected);

    core.command_handler.stop();
    core.monitor_manager.stop();
    core.connection_manager.stop();
    host_manager.borrow_mut().stop();
}

#[test]
fn updated_store_secret_is_injected_on_reconfigure() {
    let store = Arc::new(MemorySecretStore::new());
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    let mut connection_manager = ConnectionManager::new(Arc::new(stub_factory()));

    store.set(&lookup_key, "first-password").unwrap();
    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    let (runtime_secrets, _) = Configuration::prepare_host_secrets(&mut hosts, store.clone());
    connection_manager.configure(&hosts, &runtime_secrets);
    assert_eq!(
        ssh_settings(&connection_manager).get("password").map(String::as_str),
        Some("first-password")
    );

    store.set(&lookup_key, "second-password").unwrap();
    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    let (runtime_secrets, _) = Configuration::prepare_host_secrets(&mut hosts, store);
    connection_manager.configure(&hosts, &runtime_secrets);
    assert_eq!(
        ssh_settings(&connection_manager).get("password").map(String::as_str),
        Some("second-password")
    );
}

#[test]
fn non_secret_connector_settings_are_unchanged() {
    let store = Arc::new(MemorySecretStore::new());
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    store.set(&lookup_key, "stored-password").unwrap();

    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    let settings = configure_ssh(&mut hosts, store);

    assert_eq!(settings.get("username").map(String::as_str), Some(USERNAME));
    assert_eq!(hosts.hosts[HOST_ID].address.as_str(), "127.0.0.1");
}
