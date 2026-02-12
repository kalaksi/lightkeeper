/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pub fn strip_newline<Stringable: ToString>(input: &Stringable) -> String {
    let input = input.to_string();
    input.strip_suffix("\r\n").or(input.strip_suffix('\n')).unwrap_or(&input).to_string()
}

/// For a single line, keep only the content after the last carriage return.
pub fn normalize_line(line: &str) -> String {
    let line = line.trim_end_matches('\r');
    if let Some(last) = line.rfind('\r') {
        line[last + 1..].to_string()
    }
    else {
        line.to_string()
    }
}

/// For each line, keep only the content after the last carriage return so progress lines show
/// the latest update instead of concatenated overwrites. Also removes carriage returns from the end of the line.
pub fn normalize_lines(full_text: &str) -> String {
    full_text.split('\n').map(normalize_line).collect::<Vec<_>>().join("\n")
}

pub fn remove_whitespace<Stringable: ToString>(input: &Stringable) -> String {
    input.to_string().chars().filter(|&c| !c.is_whitespace()).collect()
}

pub fn get_string_between<Stringable: ToString>(input: &Stringable, start: &str, end: &str) -> Option<String> {
    let input = input.to_string();

    let start = match input.find(start) {
        Some(index) => index + start.len(),
        None => return None,
    };

    let end = match input.find(end) {
        Some(index) => index,
        None => return None,
    };

    Some(input[start..end].to_string())
}

pub fn remove_quotes<Stringable: ToString>(input: &Stringable) -> String {
    let input = input.to_string();
    input
        .strip_prefix('"')
        .or(input.strip_prefix('\''))
        .and_then(|input| input.strip_suffix('"'))
        .or(input.strip_suffix('\''))
        .unwrap_or(&input)
        .to_string()
}
