/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::BTreeMap;
use std::sync::mpsc;
use std::time::Duration;

use lightkeeper::configuration;
use lightkeeper::enums::Criticality;
use lightkeeper::error::LkError;
use lightkeeper::frontend::{DisplayOptions, UIUpdate};
use lightkeeper::host_manager::{HostManager, StateUpdateMessage};
use lightkeeper::module::monitoring::DataPoint;
use lightkeeper::module::ModuleSpecification;

const HOST_ID: &str = "test-host";
const MONITOR_ID: &str = "test-monitor";

fn setup() -> (HostManager, mpsc::Sender<StateUpdateMessage>, mpsc::Receiver<UIUpdate>) {
    let mut host_manager = HostManager::new();

    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(HOST_ID.to_string(), host_settings)]),
        ..Default::default()
    };
    host_manager.configure(&hosts_config);

    let (ui_sender, ui_receiver) = mpsc::channel();
    host_manager.add_observer(ui_sender);
    host_manager.start_receiving_updates();

    let state_sender = host_manager.new_state_update_sender();
    (host_manager, state_sender, ui_receiver)
}

fn monitor_update(
    invocation_id: u64,
    data_point: Option<DataPoint>,
    errors: Vec<LkError>,
) -> StateUpdateMessage {
    StateUpdateMessage::Monitor {
        invocation_id,
        host_name: HOST_ID.to_string(),
        display_options: DisplayOptions {
            category: String::from("test"),
            display_text: String::from("Test monitor"),
            ..Default::default()
        },
        module_spec: ModuleSpecification::monitor(MONITOR_ID, "0.0.1"),
        data_point,
        errors,
    }
}

fn next_host_update(ui_receiver: &mpsc::Receiver<UIUpdate>) -> lightkeeper::frontend::HostDisplayData {
    match ui_receiver.recv_timeout(Duration::from_secs(2)).unwrap() {
        UIUpdate::Host(display_data) => display_data,
        other => panic!("expected UIUpdate::Host, got {:?}", std::mem::discriminant(&other)),
    }
}

#[test]
fn monitor_pending_then_result() {
    let (mut host_manager, state_sender, ui_receiver) = setup();

    // Seed monitor_data entry (invocation id 0).
    state_sender.send(monitor_update(0, Some(DataPoint::pending()), Vec::new())).unwrap();
    let seeded = next_host_update(&ui_receiver);
    assert!(seeded.host_state.monitor_data.contains_key(MONITOR_ID));
    assert!(seeded.host_state.monitor_invocations.is_empty());
    assert!(seeded.new_monitoring_data.is_none());

    // Pending refresh.
    state_sender.send(monitor_update(7, Some(DataPoint::pending()), Vec::new())).unwrap();
    let pending = next_host_update(&ui_receiver);
    assert!(pending.host_state.monitor_invocations.contains_key(&7));
    assert!(pending.new_monitoring_data.is_none());

    // Completed datapoint.
    let result = DataPoint::value_with_level(String::from("42"), Criticality::Normal);
    state_sender.send(monitor_update(7, Some(result), Vec::new())).unwrap();
    let completed = next_host_update(&ui_receiver);

    assert!(completed.host_state.monitor_invocations.is_empty());
    let (invocation_id, monitoring_data) = completed.new_monitoring_data.unwrap();
    assert_eq!(invocation_id, 7);
    assert_eq!(monitoring_data.values.back().unwrap().value, "42");
    assert_eq!(
        completed.host_state.monitor_data[MONITOR_ID].values.back().unwrap().value,
        "42"
    );

    host_manager.stop();
}

#[test]
fn monitor_error_only_clears_invocation() {
    let (mut host_manager, state_sender, ui_receiver) = setup();

    state_sender.send(monitor_update(0, Some(DataPoint::pending()), Vec::new())).unwrap();
    let _ = next_host_update(&ui_receiver);

    state_sender.send(monitor_update(11, Some(DataPoint::pending()), Vec::new())).unwrap();
    let pending = next_host_update(&ui_receiver);
    assert!(pending.host_state.monitor_invocations.contains_key(&11));

    state_sender
        .send(monitor_update(11, None, vec![LkError::other("boom")]))
        .unwrap();
    let errored = next_host_update(&ui_receiver);

    assert!(errored.host_state.monitor_invocations.is_empty());
    assert_eq!(errored.new_errors.len(), 1);
    assert!(errored.new_errors[0].message.contains("boom"));
    assert!(errored.new_monitoring_data.is_none());

    host_manager.stop();
}
