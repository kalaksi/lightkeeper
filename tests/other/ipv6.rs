/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use lightkeeper::Host;
use lightkeeper::module::monitoring::network::TcpConnect;
use lightkeeper::module::monitoring::MonitoringModule;
use lightkeeper::module::Module;
use lightkeeper::utils::host_address;

#[test]
fn host_new_parses_ipv4_and_ipv6() {
    let host = Host::new("web", "2001:db8::1", "", &[]).unwrap();
    assert_eq!(host.ip_address, IpAddr::V6("2001:db8::1".parse().unwrap()));

    let host = Host::new("web", "[::1]", "", &[]).unwrap();
    assert_eq!(host.ip_address, IpAddr::V6(Ipv6Addr::LOCALHOST));

    let host = Host::new("web", "127.0.0.1", "", &[]).unwrap();
    assert_eq!(host.ip_address, IpAddr::V4(Ipv4Addr::LOCALHOST));

    let host = Host::new("web", "::", "", &[]).unwrap();
    assert!(host.ip_address.is_unspecified());
    assert!(host.ip_address.is_ipv4());

    let host = Host::new("web", "not-an-ip", "example.test", &[]).unwrap();
    assert!(host.ip_address.is_unspecified());
    assert_eq!(host.fqdn, "example.test");
}

#[test]
fn host_get_address_prefers_fqdn() {
    let host = Host::new("web", "2001:db8::1", "web.example", &[]).unwrap();
    assert_eq!(host.get_address(), "web.example");

    let host = Host::new("web", "2001:db8::1", "", &[]).unwrap();
    assert_eq!(host.get_address(), "2001:db8::1");
}

#[test]
fn format_and_split_host_port() {
    assert_eq!(host_address::format_host_port("example.test", 22), "example.test:22");
    assert_eq!(host_address::format_host_port("127.0.0.1", 22), "127.0.0.1:22");
    assert_eq!(host_address::format_host_port("2001:db8::1", 22), "[2001:db8::1]:22");
    assert_eq!(host_address::format_host_port("[::1]", 443), "[::1]:443");
    assert_eq!(
        host_address::format_ip_port(IpAddr::V6(Ipv6Addr::LOCALHOST), 22),
        "[::1]:22"
    );

    assert_eq!(host_address::split_host_port("example.test:443", "443"), ("example.test", "443"));
    assert_eq!(host_address::split_host_port("[2001:db8::1]:22", "443"), ("2001:db8::1", "22"));
    assert_eq!(host_address::split_host_port("2001:db8::1", "443"), ("2001:db8::1", "443"));
    assert_eq!(host_address::split_host_port("127.0.0.1", "443"), ("127.0.0.1", "443"));
}

#[test]
fn bracketed_host_brackets_ipv6() {
    assert_eq!(host_address::bracketed_host("host.example").unwrap(), "host.example");
    assert_eq!(host_address::bracketed_host("10.0.0.1").unwrap(), "10.0.0.1");
    assert_eq!(host_address::bracketed_host("::1").unwrap(), "[::1]");
    assert_eq!(host_address::bracketed_host("[2001:db8::1]").unwrap(), "[2001:db8::1]");
    assert!(host_address::bracketed_host("-evil").is_none());
    assert!(host_address::bracketed_host("").is_none());
}

#[test]
fn tcp_connect_message_uses_bracketed_ipv6() {
    let monitor = TcpConnect::new(&HashMap::new());
    let host = Host::new("web", "2001:db8::10", "", &[]).unwrap();
    let message = monitor.get_connector_message(host, Default::default()).unwrap();
    assert_eq!(message, "[2001:db8::10]:22");

    let host = Host::new("web", "127.0.0.1", "", &[]).unwrap();
    let message = monitor.get_connector_message(host, Default::default()).unwrap();
    assert_eq!(message, "127.0.0.1:22");
}
