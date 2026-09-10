/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use chrono::{Local, TimeZone};
use serde_json::Value;

// Match ThemeModel / journalctl priority colors for log highlighting.
const COLOR_MUTED: &str = "#a0a0a0";
const COLOR_UNIT: &str = "#c9b458";
const COLOR_ERROR: &str = "firebrick";
const COLOR_WARNING: &str = "orange";

/// Converts journalctl `-o json` output (one JSON object per line) into Qt rich text.
/// Non-JSON lines are HTML-escaped and passed through.
pub fn journalctl_json_to_rich_text(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| match serde_json::from_str::<Value>(line) {
            Ok(entry) => format_journal_entry(&entry),
            Err(_) => escape_html(line),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_journal_entry(entry: &Value) -> String {
    let timestamp = entry
        .get("__REALTIME_TIMESTAMP")
        .and_then(Value::as_str)
        .and_then(|micros| micros.parse::<i64>().ok())
        .and_then(|micros| Local.timestamp_micros(micros).single())
        .map(|dt| dt.format("%b %d %H:%M:%S").to_string())
        .unwrap_or_else(|| String::from("???"));

    let hostname = entry.get("_HOSTNAME").and_then(Value::as_str).unwrap_or("-");

    let unit = entry
        .get("SYSLOG_IDENTIFIER")
        .or_else(|| entry.get("_COMM"))
        .or_else(|| entry.get("_SYSTEMD_UNIT"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    let pid = entry.get("_PID").and_then(Value::as_str);
    let unit_with_pid = match pid {
        Some(pid) => format!("{}[{}]", unit, pid),
        None => unit.to_string(),
    };

    let message = message_from_entry(entry);
    let message_color = priority_message_color(
        entry
            .get("PRIORITY")
            .and_then(Value::as_str)
            .and_then(|priority| priority.parse::<u8>().ok()),
    );

    let mut line = String::new();
    line.push_str(&colored_span(COLOR_MUTED, &timestamp));
    line.push(' ');
    line.push_str(&colored_span(COLOR_MUTED, hostname));
    line.push(' ');
    line.push_str(&styled_span(&format!("color:{};font-weight:bold", COLOR_UNIT), &unit_with_pid));
    line.push_str(": ");
    match message_color {
        Some(color) => line.push_str(&colored_span(color, &message)),
        None => line.push_str(&escape_html(&message)),
    }
    line
}

fn message_from_entry(entry: &Value) -> String {
    match entry.get("MESSAGE") {
        Some(Value::String(message)) => message.replace(['\n', '\r'], " "),
        Some(Value::Array(bytes)) => {
            let parsed: Option<Vec<u8>> = bytes
                .iter()
                .map(|value| value.as_u64().and_then(|byte| u8::try_from(byte).ok()))
                .collect();
            match parsed.and_then(|bytes| String::from_utf8(bytes).ok()) {
                Some(message) => message.replace(['\n', '\r'], " "),
                None => String::from("[binary message]"),
            }
        }
        _ => String::new(),
    }
}

fn priority_message_color(priority: Option<u8>) -> Option<&'static str> {
    match priority {
        Some(0..=3) => Some(COLOR_ERROR),
        Some(4) => Some(COLOR_WARNING),
        Some(7) => Some(COLOR_MUTED),
        _ => None,
    }
}

fn colored_span(color: &str, text: &str) -> String {
    styled_span(&format!("color:{}", color), text)
}

fn styled_span(style: &str, text: &str) -> String {
    format!(r#"<span style="{}">{}</span>"#, style, escape_html(text))
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#039;")
}
