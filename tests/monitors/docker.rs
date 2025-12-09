use std::collections::{BTreeMap, HashMap};

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::docker;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;
use lightkeeper::configuration;

use crate::{MonitorTestHarness, StubSsh2, StubHttp, TEST_HOST_ID};


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

    harness.verify_next_datapoint(&docker::Compose::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "project1");
        assert_eq!(datapoint.multivalue[0].value, "Up 2 hours");
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

    harness.verify_next_datapoint(&docker::Containers::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
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

    harness.verify_next_datapoint(&docker::Images::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
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

    harness.verify_next_datapoint(&docker::Compose::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });

    harness.verify_next_datapoint(&docker::Containers::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });

    harness.verify_next_datapoint(&docker::Images::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });
}

#[test]
fn test_image_updates() {
    // Clear any previous HTTP stub responses and set up the response for this test
    StubHttp::clear_responses();
    // Pre-populate the shared state with the expected response
    // The returned connector is discarded, but the shared state is set up
    let _connector = StubHttp::new("https://registry.hub.docker.com/v2/repositories/library/test-service/tags/latest",
r#"{
  "name": "latest",
  "images": [
    {
      "architecture": "amd64",
      "os": "linux",
      "size": 100000000,
      "last_pushed": "2024-01-15T10:30:00.000Z"
    }
  ]
}"#);

    // Create host settings with both parent and extension monitors
    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    
    host_settings.effective.monitors.insert(
        docker::Images::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        }
    );
    
    host_settings.effective.monitors.insert(
        docker::ImageUpdates::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings: HashMap::new(),
            ..Default::default()
        }
    );
    
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default()
    );
    
    host_settings.effective.connectors.insert(
        StubHttp::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default()
    );

    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::from([(TEST_HOST_ID.to_string(), PlatformInfo::linux(Flavor::Debian, "12.0"))]),
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![
            (StubSsh2::get_metadata(), |_settings: &HashMap<String, String>| {
                let mut ssh = StubSsh2::default();
                // TODO: auto-generated, check or replace.
                ssh.add_response(r#""sudo" "curl" "-s" "--unix-socket" "/var/run/docker.sock" "http://localhost/images/json""#,
r#"[{
  "Id": "sha256:abc123",
  "Created": 1700000000,
  "RepoTags": ["test-service:latest"],
  "Size": 100000000
}]"#, 0);
                Box::new(ssh) as connection::Connector
            }),
            (StubHttp::get_metadata(), |_settings: &HashMap<String, String>| {
                // TODO: auto-generated, check or replace.
                StubHttp::new("https://registry.hub.docker.com/v2/repositories/library/test-service/tags/latest",
r#"{
  "name": "latest",
  "images": [
    {
      "architecture": "amd64",
      "os": "linux",
      "size": 100000000,
      "status": "active",
      "last_pushed": "2024-01-15T10:30:00.000Z"
    }
  ]
}"#)
            }),
        ],
        vec![
            (docker::Images::get_metadata(), docker::Images::new_monitoring_module),
            (docker::ImageUpdates::get_metadata(), docker::ImageUpdates::new_monitoring_module),
        ],
        vec![]
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);
    harness.refresh_monitors();

    // Wait for both parent and extension monitors to complete
    harness.wait_for_completion();
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Verify the extension monitor updated the parent's data
    // Extension monitors store data under their own ID, but we can check the parent's data was updated
    let display_data = harness.host_manager.borrow().get_display_data();
    let host_display = display_data.hosts.get(TEST_HOST_ID).unwrap();
    
    // Check that the extension monitor completed
    if let Some(extension_data) = host_display.host_state.monitor_data.get(&docker::ImageUpdates::get_metadata().module_spec.id) {
        let latest_datapoint = extension_data.values.back().unwrap();
        assert_eq!(latest_datapoint.multivalue.len(), 1);
        assert_eq!(latest_datapoint.multivalue[0].label, "test-service:latest");
        // The extension monitor updates the parent's multivalue value
        assert!(latest_datapoint.multivalue[0].value == "Up-to-date" || latest_datapoint.multivalue[0].value == "Outdated");
    } else {
        panic!("Extension monitor data should exist");
    }
}

