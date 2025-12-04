use std::collections::HashMap;

use lightkeeper::module::monitoring::*;
use lightkeeper::module::platform_info::*;
use lightkeeper::module::*;

use crate::{MonitorTestHarness, StubSsh2};



#[test]
fn test_uptime() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let ssh = StubSsh2::new("uptime", " 17:26:40 up 16 days,  4:25,  1 user,  load average: 0.06, 0.05, 0.01", 0);
        Box::new(ssh) as connection::Connector
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (monitoring::linux::Uptime::get_metadata(), monitoring::linux::Uptime::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&monitoring::linux::Uptime::get_metadata().module_spec.id, |datapoint| {
        // println!(harness.host)
        assert_eq!(datapoint.value, "16");
    });
}

#[test]
fn test_load() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let ssh = StubSsh2::new("uptime", " 17:26:40 up 16 days,  4:25,  1 user,  load average: 0.06, 0.05, 0.01", 0);
        Box::new(ssh) as connection::Connector
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (monitoring::linux::Load::get_metadata(), monitoring::linux::Load::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&monitoring::linux::Load::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.value, "0.06, 0.05, 0.01");
        assert_eq!(datapoint.value_float, 0.06);
    });
}


#[test]
fn test_interface() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        let ssh = StubSsh2::new("ip -j addr show", 
r#"[{"ifindex":1,"ifname":"lo","flags":["LOOPBACK","UP","LOWER_UP"],"mtu":65536,"qdisc":"noqueue",
"operstate":"UNKNOWN","group":"default","txqlen":1000,"link_type":"loopback","address":"00:00:00:00:00:00",
"broadcast":"00:00:00:00:00:00","addr_info":[{"family":"inet","local":"127.0.0.1","prefixlen":8,"scope":"host",
"label":"lo","valid_life_time":4294967295,"preferred_life_time":4294967295},{"family":"inet6","local":"::1",
"prefixlen":128,"scope":"host","noprefixroute":true,"valid_life_time":4294967295,"preferred_life_time":4294967295}]},
{"ifindex":2,"ifname":"eth0","flags":["BROADCAST","MULTICAST","UP","LOWER_UP"],"mtu":1500,"qdisc":"fq_codel",
"operstate":"UP","group":"default","txqlen":1000,"link_type":"ether","address":"00:00:00:00:00:00",
"broadcast":"ff:ff:ff:ff:ff:ff","altnames":["enp0s3","ens3"],"addr_info":[{"family":"inet","local":"1.2.3.4",
"prefixlen":32,"broadcast":"1.2.3.4","scope":"global","dynamic":true,"label":"eth0","valid_life_time":63633,
"preferred_life_time":63633}]}]"#, 0);

        Box::new(ssh) as connection::Connector
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (monitoring::linux::Interface::get_metadata(), monitoring::linux::Interface::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&monitoring::linux::Interface::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue[0].label, "lo");
        assert_eq!(datapoint.multivalue[0].multivalue[0].label, "127.0.0.1/8");
        assert_eq!(datapoint.multivalue[0].multivalue[1].label, "::1/128");
        assert_eq!(datapoint.multivalue[1].label, "eth0");
        assert_eq!(datapoint.multivalue[1].value, "up");
        assert_eq!(datapoint.multivalue[1].multivalue[0].label, "1.2.3.4/32");
    });
}

