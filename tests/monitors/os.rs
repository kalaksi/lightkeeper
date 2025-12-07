use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubSsh2};


#[test]
fn test_os() {
    // OS module doesn't use SSH, it uses platform info directly
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("", 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (monitoring::os::Os::get_metadata(), monitoring::os::Os::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&monitoring::os::Os::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.value, "Linux (Debian 12.0)");
        assert_eq!(datapoint.criticality, Criticality::Normal);
    });
}

