use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::storage;
use lightkeeper::module::platform_info::*;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubSsh2};


#[test]
fn test_filesystem() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new("df -hPT",
r#"Filesystem     Type      Size  Used Avail Use% Mounted on
/dev/sda1       ext4       20G   8.0G   11G  43% /
/dev/sda2       ext4       50G   40G    8.0G 84% /home
tmpfs           tmpfs      2.0G     0  2.0G   0% /dev/shm"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (storage::Filesystem::get_metadata(), storage::Filesystem::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&storage::Filesystem::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 2); // /dev/shm is ignored
        assert_eq!(datapoint.multivalue[0].label, "/");
        assert_eq!(datapoint.multivalue[0].value, "43 %");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
        assert_eq!(datapoint.multivalue[1].label, "/home");
        assert_eq!(datapoint.multivalue[1].value, "84 %");
        assert_eq!(datapoint.multivalue[1].criticality, Criticality::Warning);
    });
}

#[test]
fn test_cryptsetup() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new("lsblk -b -p -o NAME,SIZE,FSTYPE,MOUNTPOINT --json",
r#"{
  "blockdevices": [
    {
      "name": "/dev/sda1",
      "size": 53687091200,
      "fstype": "crypto_LUKS",
      "mountpoint": "/",
      "children": []
    }
  ]
}"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (storage::Cryptsetup::get_metadata(), storage::Cryptsetup::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&storage::Cryptsetup::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "sda1");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
    });
}

#[test]
fn test_lvm_logical_volume() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "lvs" "--separator" "|" "--options" "lv_path,lv_name,vg_name,lv_size,lv_attr,sync_percent,raid_mismatch_count,snap_percent" "--units" "h""#,
r#"  LV Path|LV|VG|LSize|Attr|Sync%|#Mis|Snap%  
  /dev/vg0/lv0|lv0|vg0|10.00h|-wi-ao----|100.00|0|"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (storage::lvm::LogicalVolume::get_metadata(), storage::lvm::LogicalVolume::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&storage::lvm::LogicalVolume::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "lv0");
        assert_eq!(datapoint.multivalue[0].value, "OK");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
    });
}

#[test]
fn test_lvm_volume_group() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "vgs" "--separator" "|" "--options" "vg_name,vg_attr,vg_size,vg_free" "--units" "h""#,
r#"  VG|Attr|VSize|VFree  
  vg0|wz--n-|20.00h|10.00h"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (storage::lvm::VolumeGroup::get_metadata(), storage::lvm::VolumeGroup::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&storage::lvm::VolumeGroup::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "vg0");
        assert_eq!(datapoint.multivalue[0].value, "OK");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
    });
}

#[test]
fn test_lvm_physical_volume() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new(r#""sudo" "pvs" "--separator" "|" "--options" "pv_name,pv_attr,pv_size,pv_free" "--units" "h""#,
r#"  PV|Attr|PSize|PFree  
  /dev/sda1|a--|20.00h|10.00h"#, 0)
    };

    let mut harness = MonitorTestHarness::new_monitor_tester(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        (storage::lvm::PhysicalVolume::get_metadata(), storage::lvm::PhysicalVolume::new_monitoring_module),
    );

    harness.refresh_monitors();

    harness.verify_monitor_data(&storage::lvm::PhysicalVolume::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "/dev/sda1");
        assert_eq!(datapoint.multivalue[0].value, "OK");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
    });
}

#[test]
/// Test handling of invalid responses of all storage-category monitors.
fn test_invalid_responses() {
    let new_stub_ssh = |_settings: &HashMap<String, String>| {
        // TODO: auto-generated responses, check or replace with actual
        StubSsh2::new_any("invalid-response", 1)
    };

    let mut harness = MonitorTestHarness::new_monitor_testers(
        PlatformInfo::linux(Flavor::Debian, "12.0"),
        (StubSsh2::get_metadata(), new_stub_ssh),
        vec![
            (storage::Filesystem::get_metadata(), storage::Filesystem::new_monitoring_module),
            (storage::Cryptsetup::get_metadata(), storage::Cryptsetup::new_monitoring_module),
            (storage::lvm::LogicalVolume::get_metadata(), storage::lvm::LogicalVolume::new_monitoring_module),
            (storage::lvm::VolumeGroup::get_metadata(), storage::lvm::VolumeGroup::new_monitoring_module),
            (storage::lvm::PhysicalVolume::get_metadata(), storage::lvm::PhysicalVolume::new_monitoring_module),
        ],
    );

    harness.refresh_monitors();

    // On error, monitors keep the initial datapoint (Normal criticality, empty multivalue)
    // or return NoData depending on how they handle errors
    harness.verify_monitor_data(&storage::Filesystem::get_metadata().module_spec.id, |datapoint| {
        assert!(datapoint.criticality == Criticality::NoData || datapoint.multivalue.is_empty());
    });

    harness.verify_monitor_data(&storage::Cryptsetup::get_metadata().module_spec.id, |datapoint| {
        assert_eq!(datapoint.multivalue.len(), 0);
    });

    harness.verify_monitor_data(&storage::lvm::LogicalVolume::get_metadata().module_spec.id, |datapoint| {
        assert!(datapoint.criticality == Criticality::NoData || datapoint.multivalue.is_empty());
    });

    harness.verify_monitor_data(&storage::lvm::VolumeGroup::get_metadata().module_spec.id, |datapoint| {
        assert!(datapoint.criticality == Criticality::NoData || datapoint.multivalue.is_empty());
    });

    harness.verify_monitor_data(&storage::lvm::PhysicalVolume::get_metadata().module_spec.id, |datapoint| {
        assert!(datapoint.criticality == Criticality::NoData || datapoint.multivalue.is_empty());
    });
}

