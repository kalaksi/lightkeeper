use std::collections::HashMap;

use lightkeeper::module::*;
use lightkeeper::module::monitoring::*;
use lightkeeper::module::monitoring::internal;
use lightkeeper::enums::Criticality;

use crate::{MonitorTestHarness, StubTcp};

const CERT_50_YEARS: &str = "-----BEGIN CERTIFICATE-----
MIIDJTCCAg2gAwIBAgIUDipWTQ9b6L/TquOABqkWYazfrRYwDQYJKoZIhvcNAQEL
BQAwITEfMB0GA1UEAwwWdGVzdDUwLmV4YW1wbGUuaW52YWxpZDAgFw0yNTEyMDcx
NjQ3NTdaGA8yMDc1MTEyNTE2NDc1N1owITEfMB0GA1UEAwwWdGVzdDUwLmV4YW1w
bGUuaW52YWxpZDCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAJ6vKePl
F3vSmJgM9PO3TSr2G2qElgbFWuGtD629jB2L16fU50U7iaYHGof51UDE9w69xnpE
ANwYOZcBKI5MiFohnt6Q78LqE+HfFKdyCbxbG/6ERu9zd+v6exguhW1DIWm9eVMd
ofbq+hMZjBO1I8uq4nPubRKTDG16i0bm39AwhV89z2Ll8YbVZEgR1Lq2P0fGU5Zh
GZeCP58vLhHBC3tybWEpLOR28dMrXlEclThr8k2rlvLTevnbsO86iK7+C3oBZqfV
xUgmPucXPicnLbmuVSaddJMg/pktWPvSR1ZGVDrYXF9LFcDTnbVx5vLHhXSx/rHG
Lmoo9incMqWPZV8CAwEAAaNTMFEwHQYDVR0OBBYEFDbHDc+3cjcMn9VfWwUoCBQm
HvgUMB8GA1UdIwQYMBaAFDbHDc+3cjcMn9VfWwUoCBQmHvgUMA8GA1UdEwEB/wQF
MAMBAf8wDQYJKoZIhvcNAQELBQADggEBAFt8N1+8rmW53ZwbvZtnWI6B9ME06V6v
+aalz6zaUxDWojU0eCwFZp0WYc0WMc+AUvw2XyInxgpqNstvO0zZ7Nk6Mn0Ceg7O
dSzicFv0p/4ZF6ueNTKps/FCNLAx1XZy/YnaFIETwdVb3IHAQJvOqcJn6HGKMMjX
RbDe2J8sqyyK7xYMcUOXSzSyLW5SOR35O35bZ9XfBlkRMoNoJr94qUE6WacXEW63
caThw//vl+MTVoLK+RDoM/mVegyrrONoX9QTY5wkOXLr1PTI1c7sgTZNCYmt/+yO
h5V+BoKTX6MAW3M6QQDXlu7jlLQ62eF7UC8VOML50XgFTBo7hlCJKgg=
-----END CERTIFICATE-----";

#[test]
fn test_valid_certificate() {
    // TODO: doesn't actually yet test the certificate validity.
    let new_stub_tcp = |_settings: &HashMap<String, String>| {
        StubTcp::new("example.invalid:443", CERT_50_YEARS)
    };

    let hosts_config = lightkeeper::configuration::Hosts {
        certificate_monitors: vec!["example.invalid:443".to_string()],
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubTcp::get_metadata(), new_stub_tcp)],
        vec![(internal::CertMonitor::get_metadata(), internal::CertMonitor::new_monitoring_module)],
        vec![],
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);

    harness.refresh_cert_monitors();

    harness.verify_next_datapoint(&internal::CertMonitor::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "example.invalid:443");
        assert!(datapoint.multivalue[0].value.contains("Expires in"));
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Normal);
    });
}

#[test]
fn test_multiple_addresses() {
    let new_stub_tcp = |_settings: &HashMap<String, String>| {
        let mut stub = StubTcp::default();
        stub.add_response("example.invalid:443", CERT_50_YEARS, 0);
        stub.add_response("test.invalid:443", CERT_50_YEARS, 0);
        stub.add_response("_", CERT_50_YEARS, 0);
        Box::new(stub) as connection::Connector
    };

    let hosts_config = lightkeeper::configuration::Hosts {
        certificate_monitors: vec!["example.invalid:443".to_string(), "test.invalid:443".to_string()],
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubTcp::get_metadata(), new_stub_tcp)],
        vec![(internal::CertMonitor::get_metadata(), internal::CertMonitor::new_monitoring_module)],
        vec![],
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);

    harness.refresh_cert_monitors();

    harness.verify_next_datapoint(&internal::CertMonitor::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 2);
        assert_eq!(datapoint.multivalue[0].label, "example.invalid:443");
        assert_eq!(datapoint.multivalue[1].label, "test.invalid:443");
    });
}

#[test]
fn test_connection_error() {
    let new_stub_tcp = |_settings: &HashMap<String, String>| {
        StubTcp::new_error("example.invalid:443", "Connection refused")
    };

    let hosts_config = lightkeeper::configuration::Hosts {
        certificate_monitors: vec!["example.invalid:443".to_string()],
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubTcp::get_metadata(), new_stub_tcp)],
        vec![(internal::CertMonitor::get_metadata(), internal::CertMonitor::new_monitoring_module)],
        vec![],
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);

    harness.refresh_cert_monitors();

    harness.verify_next_datapoint(&internal::CertMonitor::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "example.invalid:443");
        assert!(datapoint.multivalue[0].value.contains("Error:"));
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Error);
    });
}

#[test]
fn test_invalid_pem() {
    let new_stub_tcp = |_settings: &HashMap<String, String>| {
        StubTcp::new("example.invalid:443", "invalid PEM data")
    };

    let hosts_config = lightkeeper::configuration::Hosts {
        certificate_monitors: vec!["example.invalid:443".to_string()],
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubTcp::get_metadata(), new_stub_tcp)],
        vec![(internal::CertMonitor::get_metadata(), internal::CertMonitor::new_monitoring_module)],
        vec![],
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);

    harness.refresh_cert_monitors();

    harness.verify_next_datapoint(&internal::CertMonitor::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "example.invalid:443");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Error);
    });
}

#[test]
fn test_empty_response() {
    let new_stub_tcp = |_settings: &HashMap<String, String>| {
        StubTcp::new("example.invalid:443", "")
    };

    let hosts_config = lightkeeper::configuration::Hosts {
        certificate_monitors: vec!["example.invalid:443".to_string()],
        ..Default::default()
    };

    let module_factory = ModuleFactory::new_with(
        vec![(StubTcp::get_metadata(), new_stub_tcp)],
        vec![(internal::CertMonitor::get_metadata(), internal::CertMonitor::new_monitoring_module)],
        vec![],
    );

    let mut harness = MonitorTestHarness::new(hosts_config, module_factory);

    harness.refresh_cert_monitors();

    harness.verify_next_datapoint(&internal::CertMonitor::get_metadata().module_spec.id, |datapoint| {
        let datapoint = datapoint.expect("Should have datapoint");
        assert_eq!(datapoint.multivalue.len(), 1);
        assert_eq!(datapoint.multivalue[0].label, "example.invalid:443");
        assert_eq!(datapoint.multivalue[0].value, "No certificate received.");
        assert_eq!(datapoint.multivalue[0].criticality, Criticality::Error);
    });
}
