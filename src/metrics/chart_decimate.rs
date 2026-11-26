/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

use std::collections::HashMap;

use crate::metrics::lmserver::Metric;

/// Cap series size to about 2-3x a typical chart width in points.
pub const CHART_MAX_POINTS_PER_SERIES: usize = 800;

/// Downsample each monitor's metrics per label using min/max time buckets.
pub fn decimate_chart_metrics(metrics: &mut HashMap<String, Vec<Metric>>, max_points_per_series: usize) {
    for series in metrics.values_mut() {
        *series = decimate_monitor_metrics(std::mem::take(series), max_points_per_series);
    }
}

fn decimate_monitor_metrics(metrics: Vec<Metric>, max_points_per_series: usize) -> Vec<Metric> {
    if metrics.len() <= max_points_per_series {
        return metrics;
    }

    let mut label_order = Vec::new();
    let mut by_label: HashMap<String, Vec<Metric>> = HashMap::new();
    for metric in metrics {
        if !by_label.contains_key(&metric.label) {
            label_order.push(metric.label.clone());
        }
        by_label.entry(metric.label.clone()).or_default().push(metric);
    }

    let mut result = Vec::new();
    for label in label_order {
        let mut series = by_label.remove(&label).unwrap_or_default();
        series.sort_by_key(|metric| metric.time);
        result.extend(decimate_sorted_series(series, max_points_per_series));
    }
    result
}

fn decimate_sorted_series(points: Vec<Metric>, max_points: usize) -> Vec<Metric> {
    if points.len() <= max_points || max_points < 2 {
        return points;
    }

    // Two points per bucket (min and max) so peaks survive.
    let bucket_count = (max_points / 2).max(1);
    let mut result = Vec::with_capacity(max_points);
    let len = points.len();

    for bucket in 0..bucket_count {
        let start = bucket * len / bucket_count;
        let end = (bucket + 1) * len / bucket_count;
        if start >= end {
            continue;
        }

        let mut min_index = start;
        let mut max_index = start;
        for index in start + 1..end {
            if points[index].value < points[min_index].value {
                min_index = index;
            }
            if points[index].value > points[max_index].value {
                max_index = index;
            }
        }

        if min_index == max_index {
            result.push(points[min_index].clone());
        }
        else if min_index < max_index {
            result.push(points[min_index].clone());
            result.push(points[max_index].clone());
        }
        else {
            result.push(points[max_index].clone());
            result.push(points[min_index].clone());
        }
    }

    result
}
