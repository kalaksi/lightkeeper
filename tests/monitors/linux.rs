use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::linux;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubSsh2};



#[test]
fn test_interface() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new("ip -j addr show", 
r#"[{"ifindex":1,"ifname":"lo","flags":["LOOPBACK","UP","LOWER_UP"],"mtu":65536,"qdisc":"noqueue",
"operstate":"UNKNOWN","group":"default","txqlen":1000,"link_type":"loopback","address":"00:00:00:00:00:00",
"broadcast":"00:00:00:00:00:00","addr_info":[{"family":"inet","local":"127.0.0.1","prefixlen":8,"scope":"host",
"label":"lo","valid_life_time":4294967295,"preferred_life_time":4294967295},{"family":"inet6","local":"::1",
"prefixlen":128,"scope":"host","noprefixroute":true,"valid_life_time":4294967295,"preferred_life_time":4294967295}]},
{"ifindex":2,"ifname":"eth0","flags":["BROADCAST","MULTICAST","UP","LOWER_UP"],"mtu":1500,"qdisc":"fq_codel",
"operstate":"UP","group":"default","txqlen":1000,"link_type":"ether","address":"00:00:00:00:00:00",
"broadcast":"ff:ff:ff:ff:ff:ff","altnames":["enp0s3","ens3"],"addr_info":[{"family":"inet","local":"1.2.3.4",
"prefixlen":32,"broadcast":"1.2.3.4","scope":"global","dynamic":true,"label":"eth0","valid_life_time":63633,
"preferred_life_time":63633}]}]"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Interface::get_metadata(), linux::Interface::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&linux::Interface::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue[0].label, "lo");
        assert_eq!(datapoint.multivalue[0].multivalue[0].label, "127.0.0.1/8");
        assert_eq!(datapoint.multivalue[0].multivalue[1].label, "::1/128");
        assert_eq!(datapoint.multivalue[1].label, "eth0");
        assert_eq!(datapoint.multivalue[1].value, "up");
        assert_eq!(datapoint.multivalue[1].multivalue[0].label, "1.2.3.4/32");
    });
}

#[test]
fn test_kernel() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new("uname -r -m", "6.1.0-41-amd64 x86_64", 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Kernel::get_metadata(), linux::Kernel::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_next_ui_update(&linux::Kernel::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::Normal);
    });
}

#[test]
fn test_load() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new("uptime", " 17:26:40 up 16 days,  4:25,  1 user,  load average: 0.06, 0.05, 0.01", 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Load::get_metadata(), linux::Load::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&linux::Load::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.value, "0.06, 0.05, 0.01");
        assert_eq!(datapoint.value_float, 0.06);
    });
}

#[test]
fn test_package() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new(r#""sudo" "apt" "list" "--upgradable""#, 
r#"Listing... Done
docker-ce-cli/bookworm 5:29.0.3-1~debian.12~bookworm amd64 [upgradable from: 5:29.0.1-1~debian.12~bookworm]
docker-ce/bookworm 5:29.0.3-1~debian.12~bookworm amd64 [upgradable from: 5:29.0.1-1~debian.12~bookworm]"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Package::get_metadata(), linux::Package::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&linux::Package::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 2);
        assert_eq!(datapoint.multivalue[0].label, "docker-ce-cli");
        assert_eq!(datapoint.multivalue[1].label, "docker-ce");
    });
}

#[test]
fn test_ram() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new("free -m",
r#"               total        used        free      shared  buff/cache   available
Mem:            1919         736         465           0         866        1182
Swap:              0           0           0
"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Ram::get_metadata(), linux::Ram::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&linux::Ram::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.value_float as i32, 38);
        assert_eq!(datapoint.criticality, Criticality::Normal);
    });
}

#[test]
fn test_uptime() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new("uptime", " 17:26:40 up 16 days,  4:25,  1 user,  load average: 0.06, 0.05, 0.01", 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Uptime::get_metadata(), linux::Uptime::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&linux::Uptime::get_metadata().module_spec.id, |datapoint| {
        // println!(harness.host)
        assert_eq!(datapoint.value, "16");
    });
}

#[test]
fn test_who() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new("who -s", "user     pts/0        2025-12-04 20:27 (10.0.0.2)", 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (linux::Who::get_metadata(), linux::Who::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&linux::Who::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue[0].label, "user");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
    });
}

#[test]
/// Test handling of invalid responses of all linux-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        StubSsh2::new_any("probably-invalid-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (linux::Interface::get_metadata(), linux::Interface::new_monitoring_module),
            (linux::Kernel::get_metadata(), linux::Kernel::new_monitoring_module),
            (linux::Load::get_metadata(), linux::Load::new_monitoring_module),
            (linux::Package::get_metadata(), linux::Package::new_monitoring_module),
            (linux::Ram::get_metadata(), linux::Ram::new_monitoring_module),
            (linux::Uptime::get_metadata(), linux::Uptime::new_monitoring_module),
            (linux::Who::get_metadata(), linux::Who::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    // Monitors shouldn't return data points on errors.
    // There should be only the initial NoData datapoint available.
    harness.verify_monitor_data(&linux::Interface::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&linux::Kernel::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&linux::Load::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&linux::Package::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&linux::Ram::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&linux::Uptime::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });

    harness.verify_monitor_data(&linux::Who::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.criticality, Criticality::NoData);
    });
}
