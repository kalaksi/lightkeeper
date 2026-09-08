/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::collections::HashSet;

use lightkeeper::enums::Criticality;
use lightkeeper::module::monitoring::DataPoint;

#[test]
fn apply_acknowledged_entries_marks_row_and_lowers_criticality() {
    let mut root = DataPoint::empty();
    root.multivalue = vec![
        DataPoint::labeled_value_with_level("cron.service".into(), "dead".into(), Criticality::Critical),
        DataPoint::labeled_value_with_level("ssh.service".into(), "running".into(), Criticality::Normal),
    ];
    root.update_criticality_from_children();
    assert_eq!(root.criticality, Criticality::Critical);

    let acknowledged = HashSet::from([String::from("cron.service")]);
    root.apply_acknowledged_entries(&acknowledged);

    assert!(root.multivalue[0].acknowledged);
    assert!(!root.multivalue[1].acknowledged);
    assert_eq!(root.criticality, Criticality::Normal);
}

#[test]
fn apply_acknowledged_entries_supports_nested_paths() {
    let mut root = DataPoint::empty();
    let mut project = DataPoint::labeled_value_with_level("myapp".into(), "error".into(), Criticality::Error);
    project.multivalue = vec![
        DataPoint::labeled_value_with_level("web".into(), "running".into(), Criticality::Normal),
        DataPoint::labeled_value_with_level("worker".into(), "exited".into(), Criticality::Error),
    ];
    root.multivalue = vec![project];
    root.update_criticality_from_children();

    let acknowledged = HashSet::from([String::from("myapp/worker")]);
    root.apply_acknowledged_entries(&acknowledged);

    assert!(!root.multivalue[0].acknowledged);
    assert!(!root.multivalue[0].multivalue[0].acknowledged);
    assert!(root.multivalue[0].multivalue[1].acknowledged);
    assert_eq!(root.multivalue[0].criticality, Criticality::Normal);
    assert_eq!(root.criticality, Criticality::Normal);
}

#[test]
fn apply_acknowledged_entries_can_clear() {
    let mut root = DataPoint::empty();
    root.multivalue = vec![
        DataPoint::labeled_value_with_level("cron.service".into(), "dead".into(), Criticality::Critical),
    ];
    root.apply_acknowledged_entries(&HashSet::from([String::from("cron.service")]));
    assert!(root.multivalue[0].acknowledged);

    root.apply_acknowledged_entries(&HashSet::new());
    assert!(!root.multivalue[0].acknowledged);
    assert_eq!(root.criticality, Criticality::Critical);
}

#[test]
fn apply_acknowledged_entries_marks_single_value_monitor() {
    let mut point = DataPoint::value_with_level("100".into(), Criticality::Error);
    point.apply_acknowledged_entries(&HashSet::from([String::from("*")]));
    assert!(point.acknowledged);
}

#[test]
fn collect_alert_leaves_counts_only_not_ok_and_acked() {
    let mut root = DataPoint::empty();
    let mut project = DataPoint::labeled_value_with_level("myapp".into(), "error".into(), Criticality::Error);
    project.multivalue = vec![
        DataPoint::labeled_value_with_level("web".into(), "running".into(), Criticality::Normal),
        DataPoint::labeled_value_with_level("worker".into(), "exited".into(), Criticality::Error),
    ];
    root.multivalue = vec![project];

    let leaves = root.collect_alert_leaves(true, "Compose");
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].entry_id, "myapp/worker");
    assert_eq!(leaves[0].criticality, Criticality::Error);
}
