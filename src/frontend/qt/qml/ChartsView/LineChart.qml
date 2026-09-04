/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Lightkeeper 1.0
import ChartJs 1.0


Item {
    id: root
    property string title: ""
    property int yMax: 100.0
    property int yMin: 0.0
    property string yLabel: "%"
    property var chartData: []
    property bool showThresholdLines: true
    property real warningLevel: 0.0
    property real criticalLevel: 0.0
    property real timeMin: 0
    property real timeMax: 0

    property var _mainData: []

    onChartDataChanged: root._refreshSeries()
    onTimeMinChanged: root._refreshSeries()
    onTimeMaxChanged: root._refreshSeries()

    function _refreshSeries() {
        if (!root.chartData) {
            root._mainData = []
        } else {
            // Convert "t" to "x" for Chart.js time series format. Used to work, doesn't anymore.
            // TODO: should have "x" to begin with.
            var convertedData = root.chartData.map(function(item) {
                if (item.t !== undefined) {
                    return {"x": item.t, "y": item.y}
                }
                return item
            })
            root._mainData = root._clipSeriesToWindow(convertedData, root.timeMin, root.timeMax)
        }

        if (chart.jsChart !== undefined) {
            chart.animateToNewData()
        }
    }

    function _interpolateY(fromPoint, toPoint, x) {
        var span = toPoint.x - fromPoint.x
        if (span === 0) {
            return fromPoint.y
        }
        return fromPoint.y + (toPoint.y - fromPoint.y) * ((x - fromPoint.x) / span)
    }

    // Keep the visible window, but add edge points so the line continues toward
    // samples that fall just outside the selected period (and is then clipped).
    function _clipSeriesToWindow(points, timeMin, timeMax) {
        if (!points || points.length === 0) {
            return []
        }
        if (!(timeMin > 0) || !(timeMax > timeMin)) {
            return points
        }

        var sorted = points.slice()
        sorted.sort(function(left, right) {
            return left.x - right.x
        })

        var previous = null
        var next = null
        var inside = []
        for (var i = 0; i < sorted.length; i++) {
            var point = sorted[i]
            if (point.x < timeMin) {
                previous = point
            } else if (point.x > timeMax) {
                if (next === null) {
                    next = point
                }
            } else {
                inside.push(point)
            }
        }

        var result = []
        if (previous) {
            var toward = inside.length > 0 ? inside[0] : next
            if (toward) {
                result.push({x: timeMin, y: root._interpolateY(previous, toward, timeMin)})
            } else {
                return [{x: timeMin, y: previous.y}, {x: timeMax, y: previous.y}]
            }
        }

        for (var j = 0; j < inside.length; j++) {
            result.push(inside[j])
        }

        if (next) {
            var from = inside.length > 0 ? inside[inside.length - 1] : previous
            if (from) {
                result.push({x: timeMax, y: root._interpolateY(from, next, timeMax)})
            }
        } else if (result.length > 0 && result[result.length - 1].x < timeMax) {
            var last = result[result.length - 1]
            result.push({x: timeMax, y: last.y})
        }

        return result
    }

    function _thresholdDataset(level, color, label) {
        var data = root._mainData || []

        return {
            label: label,
            fill: false,
            borderColor: color,
            borderWidth: 1,
            pointRadius: 0,
            borderDash: [4, 4],
            tension: 0,
            data: [{"x": root.timeMin, "y": level}, {"x": root.timeMax, "y": level}]
        }
    }

    Chart {
        id: chart
        anchors.fill: parent
        layer.enabled: true
        chartType: "line"
        chartData: {
            var data = root._mainData || []
            var datasets = [{
                label: "",
                fill: true,
                backgroundColor: "rgba(100,200,100,0.5)",
                borderColor: "rgba(255,255,255,1.0)",
                borderWidth: 1,
                pointRadius: 1,
                tension: 0,
                data: data,
            }]
            if (root.showThresholdLines && root.warningLevel > 0.0) {
                let dataset = root._thresholdDataset(root.warningLevel, Theme.criticalityColor("warning"), "Warning")
                datasets.push(dataset)
            }
            if (root.showThresholdLines && root.criticalLevel > 0.0) {
                let dataset = root._thresholdDataset(root.criticalLevel, Theme.criticalityColor("critical"), "Critical")
                datasets.push(dataset)
            }
            return { datasets: datasets }
        }

        chartOptions: {
            return {
                maintainAspectRatio: false,
                responsive: true,
                title: {
                    display: true,
                    text: root.title,
                    fontColor: Theme.textColor,
                    padding: 5,
                    lineHeight: 1.0
                },
                tooltips: {
                    mode: "index",
                    intersect: false,
                },
                hover: {
                    mode: "nearest",
                    intersect: true
                },
                legend: {
                    display: false,
                    labels: {
                        fontColor: Theme.textColor
                    }
                },
                scales: {
                    xAxes: [{
                        display: true,
                        type: "time",
                        time: {
                            // Unix timestamp in ms.
                            parser: "x",
                            unit: "day",
                            displayFormats: {
                                day: "DD"
                            }
                        },
                        scaleLabel: {
                            display: true,
                            // labelString: "Time"
                        },
                        gridLines: {
                            display: true,
                            color: "rgba(255,255,255,0.1)"
                        },
                        ticks: {
                            maxTicksLimit: 15,
                            fontColor: Theme.textColor,
                            // Performance optimization:
                            maxRotation: 0,
                            minRotation: 0,
                            min: root.timeMin,
                            max: root.timeMax,
                        }
                    }],
                    yAxes: [{
                        display: true,
                        suggestedMin: root.yMin,
                        suggestedMax: root.yMax,
                        beginAtZero: true,
                        scaleLabel: {
                            display: true,
                            labelString: root.yLabel,
                            fontColor: Theme.textColor
                        },
                        gridLines: {
                            display: true,
                            color: "rgba(255,255,255,0.1)"
                        },
                        ticks: {
                            maxTicksLimit: 8,
                            fontColor: Theme.textColor,
                            // Performance optimization:
                            maxRotation: 0,
                            minRotation: 0,
                        }
                    }]
                }
            }
        }
    }
}