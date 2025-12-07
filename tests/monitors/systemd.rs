use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::systemd;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubSsh2};


#[test]
fn test_service() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""busctl" "--no-pager" "--json=short" "call" "org.freedesktop.systemd1" "/org/freedesktop/systemd1" "org.freedesktop.systemd1.Manager" "ListUnits""#,
r#"{
  "type": "a(ssssssouso)",
  "data": [
    [
      ["ssh.service", "OpenSSH server", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/ssh_2eservice", 0, "", "/"],
      ["nginx.service", "nginx", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/nginx_2eservice", 0, "", "/"],
      ["failed.service", "Failed service", "loaded", "failed", "dead", "", "/org/freedesktop/systemd1/unit/failed_2eservice", 0, "", "/"]
    ]
  ]
}"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::Service::get_metadata(), systemd::Service::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&systemd::Service::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 3);
        assert_eq!(datapoint.multivalue[0].label, "failed.service");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Critical);
        assert_eq!(datapoint.multivalue[1].label, "nginx.service");
        assert_eq!(datapoint.multivalue[1].value, "running");
        assert_eq!(datapoint.multivalue[1].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[2].label, "ssh.service");
        assert_eq!(datapoint.multivalue[2].value, "running");
        assert_eq!(datapoint.multivalue[2].criticality, Criticality::Normal);
    });
}

#[test]
/// Test handling of invalid responses of all systemd-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("invalid-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (systemd::Service::get_metadata(), systemd::Service::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&systemd::Service::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });
}

