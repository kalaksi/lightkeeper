use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::nixos;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubSsh2};


#[test]
fn test_rebuild_generations() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "nixos-rebuild" "list-generations" "--json" 2>/dev/null"#, 
r#"[{"generation":3,"date":"2024-01-15T10:30:00Z","nixosVersion":"23.11","kernelVersion":"6.1.0","current":true},
{"generation":2,"date":"2024-01-10T08:15:00Z","nixosVersion":"23.11","kernelVersion":"6.1.0","current":false},
{"generation":1,"date":"2024-01-05T12:00:00Z","nixosVersion":"23.11","kernelVersion":"6.1.0","current":false}]"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::NixOS, "23.11"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (nixos::RebuildGenerations::get_metadata(), nixos::RebuildGenerations::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&nixos::RebuildGenerations::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 3);
        assert_eq!(datapoint.multivalue[0].label, "#3 @ 2024-01-15 10:30:00");
        assert!(datapoint.multivalue[0].tags.contains(&"Current".to_string()));
        assert_eq!(datapoint.multivalue[1].label, "#2 @ 2024-01-10 08:15:00");
        assert!(datapoint.multivalue[1].tags.contains(&"Previous".to_string()));
    });
}

#[test]
/// Test handling of invalid responses of all nixos-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("invalid-json-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::NixOS, "23.11"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (nixos::RebuildGenerations::get_metadata(), nixos::RebuildGenerations::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&nixos::RebuildGenerations::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });
}

