/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use chrono::{DateTime, Utc};

use super::NewsItem;

/// Minimal RSS 2.0 / Atom / RDF-item parser for curated security feeds.
pub fn parse_feed(body: &str, source: &str) -> Vec<NewsItem> {
    if body.contains("<entry") {
        parse_entries(body, source, "entry", true, &["updated", "published"], &["summary", "content"])
    }
    else if body.contains("<item") {
        parse_entries(
            body,
            source,
            "item",
            false,
            &["pubDate", "dc:date", "date"],
            &["description"],
        )
    }
    else {
        Vec::new()
    }
}

const MAX_SUMMARY_CHARS: usize = 800;

fn parse_entries(
    body: &str,
    source: &str,
    item_tag: &str,
    link_attr_first: bool,
    date_tags: &[&str],
    summary_tags: &[&str],
) -> Vec<NewsItem> {
    let mut items = Vec::new();
    for block in split_tag_blocks(body, item_tag) {
        let title = text_tag(&block, "title").unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let link = if link_attr_first {
            attr_tag(&block, "link", "href").or_else(|| text_tag(&block, "link"))
        }
        else {
            text_tag(&block, "link").or_else(|| attr_tag(&block, "link", "href"))
        }
        .unwrap_or_default();
        let id = text_tag(&block, "guid")
            .or_else(|| text_tag(&block, "id"))
            .unwrap_or_else(|| link.clone());
        let published = date_tags
            .iter()
            .find_map(|tag| text_tag(&block, tag))
            .and_then(|value| parse_datetime(&value));
        let summary = summary_tags
            .iter()
            .find_map(|tag| text_tag(&block, tag))
            .map(|value| plain_summary(&value))
            .unwrap_or_default();
        items.push(NewsItem {
            id: if id.is_empty() { title.clone() } else { id },
            title,
            link,
            published,
            source: source.to_string(),
            summary,
        });
    }
    items
}

fn plain_summary(raw: &str) -> String {
    let decoded = decode_entities(&strip_cdata(raw.trim()));
    let mut with_breaks = decoded;
    for needle in ["<br>", "<br/>", "<br />", "</p>", "</div>", "</li>", "</h1>", "</h2>", "</h3>"] {
        with_breaks = replace_ascii_ci(&with_breaks, needle, "\n");
    }
    let mut plain = String::new();
    let mut in_tag = false;
    for character in with_breaks.chars() {
        if character == '<' {
            in_tag = true;
            continue;
        }
        if character == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            plain.push(character);
        }
    }
    // Collapse spaces within lines; keep newlines (trim extra blank lines).
    let collapsed = plain
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if collapsed.chars().count() > MAX_SUMMARY_CHARS {
        let truncated: String = collapsed.chars().take(MAX_SUMMARY_CHARS.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
    else {
        collapsed
    }
}

fn replace_ascii_ci(haystack: &str, pattern: &str, replacement: &str) -> String {
    let lower = haystack.to_ascii_lowercase();
    let pattern_lower = pattern.to_ascii_lowercase();
    let mut result = String::with_capacity(haystack.len());
    let mut rest = haystack;
    let mut rest_lower = lower.as_str();
    while let Some(pos) = rest_lower.find(&pattern_lower) {
        result.push_str(&rest[..pos]);
        result.push_str(replacement);
        rest = &rest[pos + pattern.len()..];
        rest_lower = &rest_lower[pos + pattern_lower.len()..];
    }
    result.push_str(rest);
    result
}

fn split_tag_blocks(body: &str, tag: &str) -> Vec<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let mut blocks = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find(&open) {
        let after_open = &rest[start..];
        let content_start = match after_open.find('>') {
            Some(index) => start + index + 1,
            None => break,
        };
        if let Some(end_rel) = rest[content_start..].find(&close) {
            let end = content_start + end_rel;
            blocks.push(rest[content_start..end].to_string());
            rest = &rest[end + close.len()..];
        }
        else {
            break;
        }
    }
    blocks
}

fn text_tag(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = block.find(&open)?;
    let after = &block[start..];
    let content_start = after.find('>')? + 1;
    let absolute_start = start + content_start;
    let end = block[absolute_start..].find(&close)? + absolute_start;
    Some(decode_entities(&strip_cdata(block[absolute_start..end].trim())))
}

fn attr_tag(block: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let start = block.find(&open)?;
    let after = &block[start..];
    let tag_end = after.find('>')?;
    let tag_text = &after[..tag_end];
    let attr_key = format!("{}=\"", attr);
    let attr_start = tag_text.find(&attr_key)? + attr_key.len();
    let attr_end = tag_text[attr_start..].find('"')? + attr_start;
    Some(decode_entities(&tag_text[attr_start..attr_end]))
}

fn strip_cdata(value: &str) -> String {
    let trimmed = value.trim();
    if let Some(inner) = trimmed.strip_prefix("<![CDATA[") {
        if let Some(inner) = inner.strip_suffix("]]>") {
            return inner.to_string();
        }
    }
    trimmed.to_string()
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    let trimmed = value.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(parsed.with_timezone(&Utc));
    }
    if let Ok(parsed) = DateTime::parse_from_rfc2822(trimmed) {
        return Some(parsed.with_timezone(&Utc));
    }
    // CERT-EU and similar use CET/CEST; chrono's RFC 2822 parser rejects named zones.
    // Try CEST before CET (CET is a suffix of CEST).
    for (name, offset) in [("CEST", "+0200"), ("CET", "+0100")] {
        if let Some(prefix) = trimmed.strip_suffix(name) {
            let with_offset = format!("{}{}", prefix, offset);
            if let Ok(parsed) = DateTime::parse_from_rfc2822(with_offset.trim()) {
                return Some(parsed.with_timezone(&Utc));
            }
        }
    }
    // RDF dc:date often uses date-only or partial ISO.
    if let Ok(parsed) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return parsed.and_hms_opt(0, 0, 0).map(|naive| naive.and_utc());
    }
    None
}
