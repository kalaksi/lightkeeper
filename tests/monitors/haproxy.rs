use std::collections::{BTreeMap, HashMap};

use lightkeeper::configuration;
use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;
use lightkeeper::HostSetting;

use crate::{MonitorTestHarness, StubSsh2, TEST_HOST_ID};


const SHOW_STAT_CSV: &str = "\
# pxname,svname,qcur,scur,slim,status,type,check_status,last_chk,addr,mode
serviceA,server1,0,19,,UP,2,L4OK,Layer4 check passed,10.0.0.2:8448,tcp
serviceA,BACKEND,0,19,26213,UP,1,,,,,tcp
serviceB,server1,0,0,,DOWN,2,L4TOUT,Layer4 timeout,10.0.0.2:6697,tcp
serviceB,BACKEND,0,0,26213,DOWN,1,,,,,tcp
serviceA,FRONTEND,,19,262122,OPEN,0,,,,,tcp
";

const EXPECTED_COMMAND: &str =
    r#""echo" "show stat" | "sudo" "nc" "-U" "/run/haproxy/admin.sock""#;

#[test]
fn test_haproxy() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(EXPECTED_COMMAND, SHOW_STAT_CSV, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (Haproxy::get_metadata(), Haproxy::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&Haproxy::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 5);
        assert_eq!(datapoint.criticality, Criticality::Critical);

        assert_eq!(datapoint.multivalue[0].label, "serviceA/server1");
        assert_eq!(datapoint.multivalue[0].value, "Up");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[0].value_float, 19.0);
        assert!(datapoint.multivalue[0].tags.contains(&String::from("Server")));
        assert!(!datapoint.multivalue[0].tags.iter().any(|tag| tag == "TCP" || tag == "HTTP"));
        assert_eq!(datapoint.multivalue[0].command_params, vec!["serviceA", "server1"]);
        assert!(datapoint.multivalue[0].description.starts_with("TCP | sessions 19"));
        assert!(datapoint.multivalue[0].description.contains("L4OK"));

        assert_eq!(datapoint.multivalue[1].label, "serviceA/BACKEND");
        assert!((datapoint.multivalue[1].value_float - (19.0 / 26213.0 * 100.0)).abs() < 0.01);

        assert_eq!(datapoint.multivalue[2].label, "serviceB/server1");
        assert_eq!(datapoint.multivalue[2].value, "Down");
        assert_eq!(datapoint.multivalue[2].criticality, Criticality::Critical);
        assert!(datapoint.multivalue[2].description.contains("L4TOUT"));

        assert_eq!(datapoint.multivalue[3].label, "serviceB/BACKEND");
        assert_eq!(datapoint.multivalue[3].criticality, Criticality::Critical);

        assert_eq!(datapoint.multivalue[4].label, "serviceA/FRONTEND");
        assert_eq!(datapoint.multivalue[4].value, "Open");
        assert!(datapoint.multivalue[4].tags.contains(&String::from("Frontend")));
        assert!((datapoint.multivalue[4].value_float - (19.0 / 262122.0 * 100.0)).abs() < 0.01);
    });
}

#[test]
fn test_haproxy_without_servers() {
    let mut settings = HashMap::new();
    settings.insert("include_servers".to_string(), "false".to_string());

    let mut host_settings = configuration::HostSettings::default();
    host_settings.address = "127.0.0.1".to_string();
    host_settings.effective.host_settings = vec![HostSetting::UseSudo];
    host_settings.effective.monitors.insert(
        Haproxy::get_metadata().module_spec.id.clone(),
        configuration::MonitorConfig {
            version: "0.0.1".to_string(),
            settings,
            ..Default::default()
        },
    );
    host_settings.effective.connectors.insert(
        StubSsh2::get_metadata().module_spec.id.clone(),
        configuration::ConnectorConfig::default(),
    );

    let hosts_config = configuration::Hosts {
        hosts: BTreeMap::from([(TEST_HOST_ID.to_string(), host_settings)]),
        predefined_platforms: BTreeMap::from([(
            TEST_HOST_ID.to_string(),
            PlatformInfo::linux(Flavor::Debian, "12.0"),
        )]),
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubSsh2::get_metadata(), |_settings: &HashMap<String, String>| {
            StubSsh2::new(EXPECTED_COMMAND, SHOW_STAT_CSV, 0)
        })],
        vec![(Haproxy::get_metadata(), Haproxy::new_monitoring_module)],
        vec![],
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);
    harness.refresh_monitors();

    harness.verify_next_datapoint(&Haproxy::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        let labels: Vec<&str> = datapoint.multivalue.iter().map(|point| point.label.as_str()).collect();
        assert_eq!(labels, vec!["serviceA/BACKEND", "serviceB/BACKEND", "serviceA/FRONTEND"]);
        assert_eq!(datapoint.criticality, Criticality::Critical);
    });
}
