use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::docker;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubSsh2};


#[test]
fn test_compose() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "curl" "-s" "--unix-socket" "/var/run/docker.sock" "http://localhost/containers/json?all=true""#,
r#"[{
  "Id": "abc123",
  "Names": ["/project1_service1"],
  "Image": "nginx:latest",
  "State": "running",
  "Status": "Up 2 hours",
  "Ports": [],
  "Labels": {
    "com.docker.compose.config-hash": "hash123",
    "com.docker.compose.project": "project1",
    "com.docker.compose.project.working_dir": "/opt/project1",
    "com.docker.compose.service": "service1"
  }
}]"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (docker::Compose::get_metadata(), docker::Compose::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&docker::Compose::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "project1");
        assert_eq!(datapoint.multivalue[0].value, "Up 2 hours"); // Compose uses container.status field
        assert_eq!(datapoint.multivalue[0].multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].multivalue[0].label, "service1");
    });
}

#[test]
fn test_containers() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "curl" "-s" "--unix-socket" "/var/run/docker.sock" "http://localhost/containers/json?all=true""#,
r#"[{
  "Id": "abc123",
  "Names": ["/container1"],
  "Image": "nginx:latest",
  "State": "running",
  "Status": "Up 2 hours",
  "Ports": [],
  "Labels": {}
}, {
  "Id": "def456",
  "Names": ["/container2"],
  "Image": "redis:latest",
  "State": "exited",
  "Status": "Exited (0) 1 hour ago",
  "Ports": [],
  "Labels": {}
}]"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (docker::Containers::get_metadata(), docker::Containers::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&docker::Containers::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 2);
        assert_eq!(datapoint.multivalue[0].label, "container1");
        assert_eq!(datapoint.multivalue[0].value, "running");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[1].label, "container2");
        assert_eq!(datapoint.multivalue[1].value, "exited");
        assert_eq!(datapoint.multivalue[1].criticality, Criticality::Normal);
    });
}

#[test]
fn test_images() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        // Using a timestamp from about 100 days ago
        StubSsh2::new(r#""sudo" "curl" "-s" "--unix-socket" "/var/run/docker.sock" "http://localhost/images/json""#,
r#"[{
  "Id": "sha256:abc123",
  "Created": 1700000000,
  "RepoTags": ["nginx:latest"],
  "Size": 100000000
}, {
  "Id": "sha256:def456",
  "Created": 1600000000,
  "RepoTags": ["redis:latest"],
  "Size": 50000000
}]"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (docker::Images::get_metadata(), docker::Images::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&docker::Images::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 2);
        assert_eq!(datapoint.multivalue[0].label, "nginx:latest");
        assert!(datapoint.multivalue[0].value.contains("days old"));
        assert_eq!(datapoint.multivalue[1].label, "redis:latest");
    });
}

#[test]
/// Test handling of invalid responses of all docker-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("invalid-json-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (docker::Compose::get_metadata(), docker::Compose::new_monitoring_module),
            (docker::Containers::get_metadata(), docker::Containers::new_monitoring_module),
            (docker::Images::get_metadata(), docker::Images::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&docker::Compose::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&docker::Containers::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&docker::Images::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });
}

