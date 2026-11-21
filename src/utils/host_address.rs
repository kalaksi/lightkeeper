/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::net::{IpAddr, SocketAddr};

use super::string_validation;

/// Formats `host:port` for socket APIs. IPv6 literals (bare or bracketed) become `[addr]:port`.
pub fn format_host_port(host: &str, port: u16) -> String {
    let host = strip_brackets(host);
    if string_validation::is_ipv6_host(host) {
        format!("[{}]:{}", host, port)
    }
    else {
        format!("{}:{}", host, port)
    }
}

pub fn format_ip_port(ip: IpAddr, port: u16) -> String {
    SocketAddr::new(ip, port).to_string()
}

/// Splits `host:port`, `[ipv6]:port`, or a bare host (uses `default_port`).
/// Unbracketed IPv6 without a port is treated as host-only.
pub fn split_host_port<'a>(value: &'a str, default_port: &'a str) -> (&'a str, &'a str) {
    if let Some(rest) = value.strip_prefix('[') {
        if let Some((host, after)) = rest.split_once(']') {
            let port = after.strip_prefix(':').filter(|port| !port.is_empty()).unwrap_or(default_port);
            return (host, port);
        }
        return (value, default_port);
    }

    match value.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') && !port.is_empty() => (host, port),
        _ => (value, default_port),
    }
}

/// Parses a bare or bracketed IP literal. Zone suffixes (`%iface`) are stripped for `IpAddr`.
pub fn parse_ip_literal(value: &str) -> Option<IpAddr> {
    let inner = strip_brackets(value);
    let address = inner.split_once('%').map(|(address, _)| address).unwrap_or(inner);
    address.parse().ok()
}

/// Host for `user@host:path` forms (IPv6 must be bracketed).
pub fn bracketed_host(address: &str) -> Option<String> {
    if string_validation::is_hostname(address) || string_validation::is_ipv4_host(address) {
        Some(address.to_string())
    }
    else if string_validation::is_bracketed_ipv6_host(address) {
        Some(address.to_string())
    }
    else if string_validation::is_ipv6_host(address) {
        Some(format!("[{}]", strip_brackets(address)))
    }
    else {
        None
    }
}

fn strip_brackets(value: &str) -> &str {
    value.strip_prefix('[').and_then(|value| value.strip_suffix(']')).unwrap_or(value)
}
