/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use lightkeeper::configuration::{
    ConfigGroup, Configuration, ConnectorConfig, HostSettings, Hosts, MonitorConfig,
};
use lightkeeper::connection_manager::ConnectionManager;
use lightkeeper::module::connection::ConnectionModule;
use lightkeeper::module::monitoring::linux::Who;
use lightkeeper::module::monitoring::MonitoringModule;
use lightkeeper::module::*;
use lightkeeper::secrets_manager::{
    secret_lookup_key, SecretStore, KEYRING_PREFIX, SECRETSFILE_PREFIX,
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

fn configure_ssh(hosts: &mut Hosts, store: Arc<dyn SecretStore>) -> HashMap<String, String> {
    let runtime_secrets = Configuration::prepare_host_secrets(hosts, store);
    let mut connection_manager = ConnectionManager::new(Arc::new(stub_factory()));
    connection_manager.configure(hosts, &runtime_secrets);
    ssh_settings(&connection_manager)
}

fn ssh_settings(connection_manager: &ConnectionManager) -> HashMap<String, String> {
    connection_manager
        .get_connector(HOST_ID, &ModuleSpecification::connector("ssh", "0.0.1"))
        .unwrap()
        .get_settings()
        .unwrap()
        .clone()
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
fn missing_secret_is_not_passed_as_placeholder() {
    let store = Arc::new(MemorySecretStore::new());
    let placeholder = password_placeholder();
    let mut hosts = hosts_with_ssh_settings(&placeholder);
    let settings = configure_ssh(&mut hosts, store);

    let password = settings.get("password").unwrap();
    assert_ne!(password, &placeholder);
}

#[test]
fn wrong_backend_placeholder_does_not_inject_store_secret() {
    let store = Arc::new(MemorySecretStore::new());
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    store.set(&lookup_key, "must-not-leak").unwrap();

    let wrong_backend = format!("{}{}", SECRETSFILE_PREFIX, lookup_key);
    let mut hosts = hosts_with_ssh_settings(&wrong_backend);
    let settings = configure_ssh(&mut hosts, store);

    let password = settings.get("password").unwrap();
    assert_ne!(password, "must-not-leak");
    assert_ne!(password, &wrong_backend);
}

#[test]
fn updated_store_secret_is_injected_on_reconfigure() {
    let store = Arc::new(MemorySecretStore::new());
    let lookup_key = secret_lookup_key("ssh", &format!("host:{}", HOST_ID), "password");
    let mut connection_manager = ConnectionManager::new(Arc::new(stub_factory()));

    store.set(&lookup_key, "first-password").unwrap();
    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    let runtime_secrets = Configuration::prepare_host_secrets(&mut hosts, store.clone());
    connection_manager.configure(&hosts, &runtime_secrets);
    assert_eq!(
        ssh_settings(&connection_manager).get("password").map(String::as_str),
        Some("first-password")
    );

    store.set(&lookup_key, "second-password").unwrap();
    let mut hosts = hosts_with_ssh_settings(&password_placeholder());
    let runtime_secrets = Configuration::prepare_host_secrets(&mut hosts, store);
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
