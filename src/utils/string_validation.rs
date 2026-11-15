/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::net::{Ipv4Addr, Ipv6Addr};

const DECIMAL_SEPARATOR: char = '.';

pub fn is_alphanumeric(string: &str) -> bool {
    // chars() also handles multibyte correctly.
    string.chars().all(|char| char.is_alphanumeric())
}

pub fn is_alphanumeric_with(string: &str, allowed_chars: &str) -> bool {
    string.chars().all(|char| char.is_alphanumeric() || allowed_chars.contains(char))
}

pub fn is_ascii_alphanumeric_with(string: &str, allowed_chars: &str) -> bool {
    string
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || allowed_chars.contains(char))
}

pub fn begins_with_dash(string: &str) -> bool {
    string.chars().next().unwrap_or(' ') == '-'
}

/// SSH / rsync username aligned with `useradd(8)`: ASCII letters/digits/`_`/`-`,
/// optional trailing `$`, no leading `-`, not fully numeric, not `.` / `..`.
pub fn is_ssh_username(value: &str) -> bool {
    if value.is_empty() || begins_with_dash(value) || value == "." || value == ".." {
        return false;
    }
    let body = value.strip_suffix('$').unwrap_or(value);
    if body.is_empty() || body.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    is_ascii_alphanumeric_with(body, "_-")
}

/// DNS-like hostname (ASCII); not an IP literal.
pub fn is_hostname(value: &str) -> bool {
    !value.is_empty() && !begins_with_dash(value) && is_ascii_alphanumeric_with(value, ".-_") && !is_ipv4_host(value)
}

pub fn is_ipv4_host(value: &str) -> bool {
    value.parse::<Ipv4Addr>().is_ok()
}

/// Bare or bracketed IPv6 host, optionally with a `%zone` / interface suffix.
pub fn is_ipv6_host(value: &str) -> bool {
    let inner = match value.strip_prefix('[').and_then(|value| value.strip_suffix(']')) {
        Some(inner) => inner,
        None => value,
    };
    let (address, zone) = match inner.split_once('%') {
        Some((address, zone)) => (address, Some(zone)),
        None => (inner, None),
    };
    if address.parse::<Ipv6Addr>().is_err() {
        return false;
    }
    match zone {
        None => true,
        Some(zone) => {
            !zone.is_empty()
                && !zone.contains(|c: char| matches!(c, ':' | '@' | '/' | '%' | '[' | ']') || c.is_whitespace() || c.is_control())
        }
    }
}

/// `[IPv6]` / `[IPv6%zone]` as required in rsync `user@host:path` specs.
pub fn is_bracketed_ipv6_host(value: &str) -> bool {
    value.starts_with('[') && value.ends_with(']') && is_ipv6_host(value)
}

pub fn is_ip_address(value: &str) -> bool {
    is_ipv4_host(value) || is_ipv6_host(value)
}

/// Rsync path after `user@host:`. Deny option/daemon prefixes and whitespace/controls;
/// allow other path bytes (spaces already rejected) so real filenames keep working.
pub fn is_rsync_remote_path(value: &str) -> bool {
    !value.is_empty()
        && !begins_with_dash(value)
        && !value.starts_with(':')
        && !value.contains(|c: char| c.is_whitespace() || c.is_control())
}

pub fn is_numeric(string: &str) -> bool {
    string.parse::<f64>().is_ok()
}

// Numeric value with unit (e.g. 100M, 1.5GB, 5 m). May contain a space between number and unit.
pub fn is_numeric_with_unit(string: &str, valid_units: &[String]) -> bool {
    let value_chars = string
        .chars()
        .take_while(|char| char.is_numeric() || *char == DECIMAL_SEPARATOR)
        .collect::<String>();
    if !is_numeric(&value_chars) {
        return false;
    }

    let unit_chars = string
        .chars()
        .skip_while(|char| char.is_numeric() || *char == DECIMAL_SEPARATOR)
        .collect::<String>();
    let unit_string = unit_chars.trim().to_string();
    !unit_string.is_empty() && valid_units.contains(&unit_string)
}
