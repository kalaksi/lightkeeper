/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Theme
import ChartJs 1.0


Item {
    id: root
    property string title: ""
    property int yMax: 100.0
    property int yMin: 0.0
    property string yLabel: "%"
    property var chartData: []
    property real warningLevel: NaN
    property real criticalLevel: NaN

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
        var xMin = data.length > 0 ? data[0].x : 0
        var xMax = data.length > 0 ? data[data.length - 1].x : 0
        return {
            label: label,
            fill: false,
            borderColor: color,
            borderWidth: 1,
            pointRadius: 0,
            borderDash: [4, 4],
            data: [{"x": xMin, "y": level}, {"x": xMax, "y": level}]
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
                data: data,
            }]
            if (typeof root.warningLevel === "number" && !isNaN(root.warningLevel)) {
                datasets.push(root._thresholdDataset(
                    root.warningLevel, Theme.criticalityColor("warning"), "Warning"))
            }
            if (typeof root.criticalLevel === "number" && !isNaN(root.criticalLevel)) {
                datasets.push(root._thresholdDataset(
                    root.criticalLevel, Theme.criticalityColor("critical"), "Critical"))
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
                            maxTicksLimit: 12,
                            fontColor: Theme.textColor,
                            // Performance optimization:
                            maxRotation: 0,
                            minRotation: 0,
                        }
                    }],
                    yAxes: [{
                        display: true,
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
                            min: root.yMin,
                            max: root.yMax,
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