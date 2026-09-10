/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use lightkeeper::utils::journalctl_json_to_rich_text;

#[test]
fn formats_json_entry_fields() {
    let input = concat!(
        r#"{"PRIORITY":"3","_HOSTNAME":"host1","SYSLOG_IDENTIFIER":"sshd","_PID":"1234","#,
        r#""__REALTIME_TIMESTAMP":"1733047200000000","MESSAGE":"Failed password for root"}"#,
    );
    let output = journalctl_json_to_rich_text(input);
    assert!(output.contains("host1"));
    assert!(output.contains("sshd[1234]"));
    assert!(output.contains("Failed password for root"));
    assert!(output.contains("color:firebrick"));
    assert!(output.contains("color:#c9b458"));
    assert!(output.contains("font-weight:bold"));
    assert!(output.contains("color:#a0a0a0"));
}

#[test]
fn leaves_info_message_untinted() {
    let input = concat!(
        r#"{"PRIORITY":"6","_HOSTNAME":"host1","SYSLOG_IDENTIFIER":"systemd","#,
        r#""__REALTIME_TIMESTAMP":"1733047200000000","MESSAGE":"Started service"}"#,
    );
    let output = journalctl_json_to_rich_text(input);
    assert!(output.contains("Started service"));
    assert!(!output.contains("color:firebrick"));
    assert!(!output.contains("color:orange"));
    assert!(!output.contains(r#"color:#a0a0a0">Started service"#));
}

#[test]
fn escapes_html_in_message() {
    let input = concat!(
        r#"{"PRIORITY":"6","_HOSTNAME":"host1","SYSLOG_IDENTIFIER":"app","#,
        r#""__REALTIME_TIMESTAMP":"1733047200000000","MESSAGE":"a <b> & c"}"#,
    );
    let output = journalctl_json_to_rich_text(input);
    assert!(output.contains("a &lt;b&gt; &amp; c"));
}

#[test]
fn passes_through_non_json_lines() {
    let output = journalctl_json_to_rich_text("not json <tag>");
    assert_eq!(output, "not json &lt;tag&gt;");
}
