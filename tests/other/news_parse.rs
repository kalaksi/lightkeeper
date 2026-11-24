/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use lightkeeper::news::parse_feed;

#[test]
fn parses_rss_item() {
    let body = r#"<?xml version="1.0"?>
        <rss><channel>
        <item>
          <title>Example Advisory</title>
          <link>https://example.invalid/a</link>
          <guid>https://example.invalid/a</guid>
          <pubDate>Mon, 01 Jan 2024 12:00:00 +0000</pubDate>
        </item>
        </channel></rss>"#;
    let items = parse_feed(body, "Test");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Example Advisory");
    assert_eq!(items[0].link, "https://example.invalid/a");
    assert!(items[0].published.is_some());
}

#[test]
fn parses_atom_entry() {
    let body = r#"<?xml version="1.0"?>
        <feed>
          <entry>
            <title>Atom Title</title>
            <link href="https://example.invalid/b"/>
            <id>urn:example:b</id>
            <updated>2024-02-03T04:05:06Z</updated>
            <summary type="html">&lt;p&gt;Overview text&lt;/p&gt;</summary>
          </entry>
        </feed>"#;
    let items = parse_feed(body, "AtomSrc");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Atom Title");
    assert_eq!(items[0].link, "https://example.invalid/b");
    assert_eq!(items[0].source, "AtomSrc");
    assert_eq!(items[0].summary, "Overview text");
}

#[test]
fn parses_rss_description_as_summary() {
    let body = r#"<?xml version="1.0"?>
        <rss><channel>
        <item>
          <title>With Description</title>
          <link>https://example.invalid/c</link>
          <description>Plain summary text</description>
        </item>
        </channel></rss>"#;
    let items = parse_feed(body, "Test");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].summary, "Plain summary text");
}

#[test]
fn preserves_summary_newlines_from_html() {
    let body = r#"<?xml version="1.0"?>
        <feed>
          <entry>
            <title>Multiline</title>
            <link href="https://example.invalid/d"/>
            <id>urn:example:d</id>
            <summary type="html">&lt;p&gt;Overview&lt;/p&gt;&lt;p&gt;Line two&lt;br&gt;Line three&lt;/p&gt;</summary>
          </entry>
        </feed>"#;
    let items = parse_feed(body, "AtomSrc");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].summary, "Overview\nLine two\nLine three");
}
