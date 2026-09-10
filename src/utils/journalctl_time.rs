/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use chrono::NaiveDateTime;

/// Validates a journalctl `--since` / `--until` time argument.
/// Accepts empty, `now`, `today`, `yesterday`, relative forms like `-15min`/`-1h`/`-1day`,
/// and `%Y-%m-%d %H:%M:%S` with optional ` UTC` suffix.
pub fn is_valid_journalctl_time(value: &str) -> bool {
    if value.is_empty() || matches!(value, "now" | "today" | "yesterday") {
        return true;
    }

    let absolute = value.strip_suffix(" UTC").unwrap_or(value);
    if NaiveDateTime::parse_from_str(absolute, "%Y-%m-%d %H:%M:%S").is_ok() {
        return true;
    }

    is_relative_journalctl_time(value)
}

fn is_relative_journalctl_time(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('-')
    else {
        return false;
    };

    let digit_end = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digit_end == 0 || digit_end == rest.len() {
        return false;
    }

    matches!(
        &rest[digit_end..],
        "s" | "sec" |
            "secs" |
            "second" |
            "seconds" |
            "m" |
            "min" |
            "mins" |
            "minute" |
            "minutes" |
            "h" |
            "hr" |
            "hrs" |
            "hour" |
            "hours" |
            "d" |
            "day" |
            "days" |
            "w" |
            "week" |
            "weeks"
    )
}
