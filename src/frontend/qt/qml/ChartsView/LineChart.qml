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

    onChartDataChanged: {
        if (root.chartData) {
            setData(root.chartData)
        }
    }

    function setData(data) {
        if (!data) {
            root._mainData = []
            if (chart.jsChart !== undefined) {
                chart.animateToNewData()
            }
            return
        }

        // Convert "t" to "x" for Chart.js time series format. Used to work, doesn't anymore.
        // TODO: should have "x" to begin with.
        var convertedData = data.map(function(item) {
            if (item.t !== undefined) {
                return {"x": item.t, "y": item.y}
            }
            return item
        })
        root._mainData = convertedData

        if (chart.jsChart !== undefined) {
            chart.animateToNewData()
        }
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