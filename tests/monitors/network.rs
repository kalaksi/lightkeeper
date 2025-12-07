use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::network;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubLocalCommand, StubSsh2};


#[test]
fn test_ping() {
    let new_stub_local = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubLocalCommand::new(r#""ping" "-c" "2" "-W" "10" "127.0.0.1""#,
r#"PING 127.0.0.1 (127.0.0.1) 56(84) bytes of data.
64 bytes from 127.0.0.1: icmp_seq=1 time=0.123 ms
64 bytes from 127.0.0.1: icmp_seq=2 time=0.145 ms

--- 127.0.0.1 ping statistics ---
2 packets transmitted, 2 received, 0% packet loss, time 1000ms
rtt min/avg/max/mdev = 0.123/0.134/0.145/0.011 ms"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubLocalCommand::get_metadata(), new_stub_local),
        (network::Ping::get_metadata(), network::Ping::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&network::Ping::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.value, "0.134");
        assert_eq!(datapoint.criticality, Criticality::Normal);
    });
}

#[test]
fn test_ssh() {
    // SSH monitor doesn't use SSH connector, it just checks if platform is set
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("", 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (network::Ssh::get_metadata(), network::Ssh::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&network::Ssh::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.value, "up");
        assert_eq!(datapoint.criticality, Criticality::Normal);
    });
}

#[test]
fn test_routes() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new("ip route ls",
r#"default via 192.168.1.1 dev eth0 proto static
10.0.0.0/8 dev eth1 proto kernel scope link src 10.0.0.1
192.168.1.0/24 dev eth0 proto kernel scope link src 192.168.1.100"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (network::Routes::get_metadata(), network::Routes::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&network::Routes::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 3);
        assert_eq!(datapoint.multivalue[0].label, "default via 192.168.1.1");
        assert_eq!(datapoint.multivalue[0].value, "eth0");
        assert_eq!(datapoint.multivalue[1].label, "10.0.0.0/8");
        assert_eq!(datapoint.multivalue[1].value, "eth1");
    });
}

#[test]
fn test_dns() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let mut ssh = StubSsh2::default();
        ssh.add_response(r#""grep" "-E" "^nameserver" "/etc/resolv.conf""#, "#test\nnameserver 127.0.0.1\nnameserver 127.0.0.2", 0);
        ssh.add_response("resolvectl dns", r#"Global:\nLink 2 (enp123s0f3u1u2): 127.0.0.1\nLink 3 (wlp1s0):"#, 0);
        
        Box::new(ssh) as connection::Connector
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (network::Dns::get_metadata(), network::Dns::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_datapoint(&network::Dns::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 2);
        assert_eq!(datapoint.multivalue[0].label, "127.0.0.1");
        assert_eq!(datapoint.multivalue[1].label, "127.0.0.2");
    });
}

#[test]
/// Test handling of invalid responses of all network-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("invalid-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (network::Routes::get_metadata(), network::Routes::new_monitoring_module),
            (network::Dns::get_metadata(), network::Dns::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    // TODO: better handling of ordering, now refreshes monitors by category.
    harness.verify_next_datapoint(&network::Dns::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        // Normally doesn't return Err, but possibly empty datapoint with Critical criticality.
        assert_eq!(datapoint.criticality, Criticality::Critical);
    });

    harness.verify_next_datapoint(&network::Routes::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.is_none(), true);
    });



    //
    // For local-command connectors
    //

    let new_stub_local = |_settings: &HashMap<String, String>| {
        StubLocalCommand::new_any("invalid-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubLocalCommand::get_metadata(), new_stub_local),
        vec![
            (network::Ping::get_metadata(), network::Ping::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    // Ping is a bit special as it displays errors directly as datapoint values.
    harness.verify_next_datapoint(&network::Ping::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.criticality, Criticality::Critical);
    });
}

