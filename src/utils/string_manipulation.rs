/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use regex::Regex;

pub fn strip_newline<Stringable: ToString>(input: &Stringable) -> String {
    let input = input.to_string();
    input.strip_suffix("\r\n").or(input.strip_suffix('\n')).unwrap_or(&input).to_string()
}

/// For a single line, keep only the content after the last carriage return.
/// Also trims trailing carriage return.
pub fn normalize_line(line: &str) -> String {
    let line = line.trim_end_matches('\r');
    if let Some(last) = line.rfind('\r') {
        line[last + 1..].to_string()
    }
    else {
        line.to_string()
    }
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

/// Parse ANSI SGR codes and return Qt rich text (HTML) with appropriate styling.
/// Other ANSI sequences (cursor movement, mode changes, etc.) are stripped.
pub fn ansi_to_rich_text(text: &str) -> String {
    let strip_other = Regex::new(r"\x1b\[[\x20-\x3f]*([\x40-\x7e])").unwrap();
    let without_other: String = strip_other
        .replace_all(text, |caps: &regex::Captures<'_>| {
            if caps.get(1).map(|m| m.as_str()) == Some("m") {
                caps.get(0).unwrap().as_str().to_string()
            }
            else {
                String::new()
            }
        })
        .into_owned();

    let sgr = Regex::new(r"\x1b\[([\d;]+)m").unwrap();
    sgr.replace_all(&without_other, |caps: &regex::Captures<'_>| {
        let codes: Vec<u8> = caps.get(1).unwrap().as_str().split(';').filter_map(|s| s.parse().ok()).collect();

        if codes.contains(&0) {
            return "</span>".to_string();
        }

        let mut styles: Vec<&'static str> = Vec::new();
        for &code in &codes {
            match code {
                1 => styles.push("font-weight:bold"),
                3 => styles.push("font-style:italic"),
                4 => styles.push("text-decoration:underline"),
                9 => styles.push("text-decoration:line-through"),
                30 => styles.push("color:black"),
                31 => styles.push("color:red"),
                32 => styles.push("color:green"),
                33 => styles.push("color:yellow"),
                34 => styles.push("color:blue"),
                35 => styles.push("color:magenta"),
                36 => styles.push("color:cyan"),
                37 => styles.push("color:white"),
                90 => styles.push("color:grey"),
                _ => {}
            }
        }

        if styles.is_empty() {
            String::new()
        }
        else {
            format!(r#"<span style="{}">"#, styles.join(";"))
        }
    })
    .into_owned()
}
