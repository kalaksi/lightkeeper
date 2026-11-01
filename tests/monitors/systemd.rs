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
        StubSsh2::new(r#"'busctl' '--no-pager' '--json=short' 'call' 'org.freedesktop.systemd1' '/org/freedesktop/systemd1' 'org.freedesktop.systemd1.Manager' 'ListUnits'"#,
r#"{
  "type": "a(ssssssouso)",
  "data": [
    [
      ["ssh.service", "OpenSSH server", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/ssh_2eservice", 0, "", "/"],
      ["nginx.service", "nginx", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/nginx_2eservice", 0, "", "/"],
      ["oneshot.service", "Oneshot service", "loaded", "inactive", "dead", "", "/org/freedesktop/systemd1/unit/oneshot_2eservice", 0, "", "/"],
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
        assert_eq!(datapoint.multivalue.len(), 4);
        assert_eq!(datapoint.multivalue[0].label, "failed.service");
        assert_eq!(datapoint.multivalue[0].value, "dead (failed)");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Critical);
        assert_eq!(datapoint.multivalue[1].label, "nginx.service");
        assert_eq!(datapoint.multivalue[1].value, "running");
        assert_eq!(datapoint.multivalue[1].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[2].label, "oneshot.service");
        assert_eq!(datapoint.multivalue[2].value, "dead");
        assert_eq!(datapoint.multivalue[2].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[3].label, "ssh.service");
        assert_eq!(datapoint.multivalue[3].value, "running");
        assert_eq!(datapoint.multivalue[3].criticality, Criticality::Normal);
    });
}

#[test]
fn test_user_service() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(
            concat!(
                "'busctl' '--no-pager' '--user' '--json=short' 'call' 'org.freedesktop.systemd1' ",
                "'/org/freedesktop/systemd1' 'org.freedesktop.systemd1.Manager' 'ListUnits'"
            ),
r#"{
  "type": "a(ssssssouso)",
  "data": [
    [
      ["lightkeeper-core.service", "Lightkeeper core", "loaded", "active", "running", "", "/org/freedesktop/systemd1/unit/lightkeeper_2dcore_2eservice", 0, "", "/"],
      ["oneshot-user.service", "Oneshot user service", "loaded", "inactive", "dead", "", "/org/freedesktop/systemd1/unit/oneshot_2duser_2eservice", 0, "", "/"],
      ["failed-user.service", "Failed user service", "loaded", "failed", "dead", "", "/org/freedesktop/systemd1/unit/failed_2duser_2eservice", 0, "", "/"]
    ]
  ]
}"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::User::get_metadata(), systemd::User::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&systemd::User::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 3);
        assert_eq!(datapoint.multivalue[0].label, "failed-user.service");
        assert_eq!(datapoint.multivalue[0].value, "dead (failed)");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Critical);
        assert_eq!(datapoint.multivalue[1].label, "lightkeeper-core.service");
        assert_eq!(datapoint.multivalue[1].value, "running");
        assert_eq!(datapoint.multivalue[1].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[2].label, "oneshot-user.service");
        assert_eq!(datapoint.multivalue[2].value, "dead");
        assert_eq!(datapoint.multivalue[2].criticality, Criticality::Normal);
    });
}

#[test]
fn test_user_service_no_session() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new_any("Failed to connect to bus: No medium found", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (systemd::User::get_metadata(), systemd::User::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&systemd::User::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.criticality, Criticality::NotAvailable);
        assert_eq!(datapoint.value, "SystemD user session not available");
    });
}

#[test]
/// Test handling of invalid responses of all systemd-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new_any("invalid-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (systemd::Service::get_metadata(), systemd::Service::new_monitoring_module),
            (systemd::User::get_metadata(), systemd::User::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&systemd::Service::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });
    harness.verify_next_datapoint(&systemd::User::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });
}

