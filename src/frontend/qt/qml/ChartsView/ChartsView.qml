/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import ".."
import "../Misc"
import "../js/TextTransform.js" as TextTransform
import "../StyleOverride"


Item {
    id: root
    property string hostId: ""
    property int chartHeight: 180
    property int columnSpacing: Theme.spacingNormal
    property bool enableShortcuts: false
    property var _categories: []
    // Monitor payloads keyed by category. Rebuilt on refresh so charts appear after
    // host monitoring data arrives (common on first remote-core connect).
    property var _monitorsByCategory: ({})
    property bool _showEmptyCategories: true

    property int _periodSeconds: 7 * 24 * 60 * 60
    property int _chartStartTimeSec: 0
    property int _chartEndTimeSec: 0
    // Extra lookback so charts can continue the line toward samples just
    // outside the selected period.
    property int _chartLookbackSeconds: 7 * 24 * 60 * 60

    signal refreshRequested()

    Component.onCompleted: {
        if (root.hostId !== "") {
            // TODO: enable more categories later after better support.
            // root._categories = LK.metrics.getCategories(root.hostId)
            root._categories = ["host", "storage"]
        }
        root.rebuildMonitors()
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: rootScrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root._categories.length === 0
        scale: 1.5
        text: "Loading..."
    }

    ToolBar {
        id: chartsToolBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 36

        background: BorderRectangle {
            backgroundColor: Theme.backgroundColor
            borderColor: Theme.borderColor
            borderBottom: 1
        }

        RowLayout {
            width: parent.width
            height: parent.height
            anchors.top: parent.top
            spacing: Theme.spacingNormal

            Row {
                spacing: Theme.spacingNormal
                Layout.alignment: Qt.AlignVCenter

                Text {
                    text: "Threshold lines"
                    color: Theme.textColor
                    font.pointSize: Theme.fontSize
                    anchors.verticalCenter: parent.verticalCenter
                }

                Switch {
                    id: thresholdLinesSwitch
                    checked: LK.config.showChartThresholdLines
                    onCheckedChanged: {
                        LK.config.showChartThresholdLines = thresholdLinesSwitch.checked
                    }
                }
            }

            Item {
                Layout.fillWidth: true
            }

            Text {
                text: "Time period"
                color: Theme.textColor
                font.pointSize: Theme.fontSize
                Layout.alignment: Qt.AlignVCenter
            }

            ComboBox {
                id: periodComboBox
                model: ["24 hours", "7 days", "14 days", "30 days"]
                property var _periodSecondsList: [
                    24 * 60 * 60,
                    7 * 24 * 60 * 60,
                    14 * 24 * 60 * 60,
                    30 * 24 * 60 * 60,
                ]
                currentIndex: 1
                onCurrentIndexChanged: {
                    root._periodSeconds = _periodSecondsList[currentIndex]
                    if (root.hostId !== "") {
                        root.refreshContent()
                    }
                }
                Component.onCompleted: {
                    root._periodSeconds = _periodSecondsList[currentIndex]
                }
                font.pointSize: Theme.fontSize
                Layout.preferredWidth: 120
                Layout.alignment: Qt.AlignVCenter
            }
        }
    }

    ScrollView {
        id: rootScrollView
        anchors.top: chartsToolBar.bottom
        anchors.topMargin: Theme.spacingNormal
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        contentWidth: availableWidth
        clip: true

        GridLayout {
            id: grid
            // columns: Math.floor(parent.width / root.columnMinimumWidth)
            columns: 1
            columnSpacing: root.columnSpacing
            width: parent.width

            Repeater {
                model: root._categories

                GroupBox {
                    id: groupBox
                    required property var modelData
                    property alias categoryName: groupBox.modelData
                    property var _invocationIdToButton: ({})

                    leftPadding: Theme.spacingTight
                    rightPadding: Theme.spacingTight
                    Layout.fillWidth: true
                    Layout.preferredHeight: chartColumn.height
                    Layout.alignment: Qt.AlignTop

                    background: Rectangle {
                        color: Theme.categoryBackgroundColor
                    }

                    label: GroupBoxLabel {
                        id: groupBoxLabel
                        anchors.left: groupBox.left
                        anchors.right: groupBox.right

                        text: TextTransform.capitalize(groupBox.categoryName)
                        icon: Theme.categoryIcon(groupBox.categoryName)
                        color: Theme.categoryColor(groupBox.categoryName)
                    }

                    Grid {
                        id: chartColumn
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        spacing: 0
                        columns: 2

                        Repeater {
                            model: root._monitorsByCategory[groupBox.categoryName] || []

                            Item {
                                id: chart
                                required property var modelData
                                property var monitoringData: modelData
                                property int invocationId: -1
                                // Array of chart data.
                                // Single array for single value charts, multiple arrays for multivalue charts.
                                property var chartDatas: []

                                width: chart.monitoringData.display_options.use_multivalue ? chartColumn.width : chartColumn.width / 2
                                height: chartGrid.height

                                Connections {
                                    target: root

                                    function onRefreshRequested() {
                                        let startTimeSec = Math.max(0, root._chartStartTimeSec - root._chartLookbackSeconds)
                                        chart.invocationId = LK.metrics.refreshCharts(
                                            root.hostId,
                                            chart.monitoringData.monitor_id,
                                            startTimeSec,
                                            root._chartEndTimeSec)
                                    }
                                }

                                Connections {
                                    target: LK.metrics

                                    function onDataReceived(invocationId, chartDataJson) {
                                        if (invocationId === chart.invocationId) {
                                            let chartData = JSON.parse(chartDataJson)[chart.monitoringData.monitor_id]
                                            if (chartData === undefined || chartData.length === 0) {
                                                return
                                            }

                                            let nextDatas = []
                                            if (chart.monitoringData.display_options.use_multivalue) {
                                                let labeledData = {};
                                                let validData = chartData.filter(item => item.label !== "")

                                                for (const data of validData) {
                                                    if (labeledData[data.label] === undefined) {
                                                        labeledData[data.label] = [];
                                                    }
                                                    labeledData[data.label].push({"t": data.time * 1000, "y": data.value});
                                                }

                                                nextDatas = Object.keys(labeledData).map(label => {
                                                    return {
                                                        label: chart.monitoringData.display_options.display_text + ": " + label,
                                                        data: labeledData[label]
                                                    }
                                                });
                                            }
                                            else {
                                                let newValues = chartData.map(item => { return {"t": item.time * 1000, "y": item.value} })
                                                nextDatas = [{
                                                    label: chart.monitoringData.display_options.display_text,
                                                    data: newValues
                                                }]
                                            }

                                            chart.applySeriesData(nextDatas)

                                            // console.log("Chart data for " + chart.monitoringData.display_options.display_text + ":" + JSON.stringify(chart.chartDatas))
                                        }
                                    }
                                }

                                Grid {
                                    id: chartGrid
                                    width: parent.width
                                    height: (root.chartHeight + Theme.spacingNormal) * rowCount
                                    columns: 2
                                    columnSpacing: Theme.spacingNormal

                                    property int rowCount: Math.ceil(chart.chartDatas.length / 2)

                                    Repeater {
                                        id: seriesRepeater
                                        model: chart.chartDatas

                                        LineChart {
                                            required property var modelData
                                            required property int index

                                            chartData: modelData.data
                                            title: modelData.label
                                            width: chartColumn.width / 2.05
                                            height: root.chartHeight
                                            yLabel: chart.monitoringData.display_options.unit
                                            yMin: chart.monitoringData.display_options.value_min
                                            yMax: chart.monitoringData.display_options.value_max
                                            showThresholdLines: LK.config.showChartThresholdLines
                                            warningLevel: chart.monitoringData.charts_warning_value
                                            criticalLevel: chart.monitoringData.charts_critical_value
                                            timeMin: root._chartStartTimeSec * 1000
                                            timeMax: root._chartEndTimeSec * 1000
                                        }
                                    }
                                }

                                // Keep LineChart delegates alive on refresh so Chart create animation
                                // only runs on first paint (or when series count changes), and to avoid
                                // the cost of destroying/recreating Chart.js canvases.
                                function applySeriesData(nextDatas) {
                                    if (chart.chartDatas.length === nextDatas.length && nextDatas.length > 0) {
                                        for (let i = 0; i < nextDatas.length; i++) {
                                            let lineChart = seriesRepeater.itemAt(i)
                                            if (lineChart !== null) {
                                                lineChart.title = nextDatas[i].label
                                                lineChart.chartData = nextDatas[i].data
                                            }
                                        }
                                    } else {
                                        chart.chartDatas = nextDatas
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    function rebuildMonitors() {
        let monitorsByCategory = {}
        for (const categoryName of root._categories) {
            monitorsByCategory[categoryName] = LK.metrics.getCategoryMonitorIds(root.hostId, categoryName)
                .map(monitorId => JSON.parse(LK.hosts.getMonitoringDataJson(root.hostId, monitorId)))
                .filter(monitor => monitor.display_options.use_with_charts)
        }

        let changed = root._categories.some(categoryName => {
            let nextIds = (monitorsByCategory[categoryName] || []).map(monitor => monitor.monitor_id).join(",")
            let previousIds = (root._monitorsByCategory[categoryName] || []).map(monitor => monitor.monitor_id).join(",")
            return nextIds !== previousIds
        })
        if (changed) {
            root._monitorsByCategory = monitorsByCategory
        }
    }

    function refreshContent() {
        let endSec = Math.floor(Date.now() / 1000)
        root._chartEndTimeSec = endSec
        root._chartStartTimeSec = endSec - root._periodSeconds
        console.log("Refreshing charts for host: " + root.hostId)
        root.rebuildMonitors()
        root.refreshRequested()
    }

    function activate() {
        root.enableShortcuts = true
        // Keep any already-drawn charts visible; refresh on the next tick.
        Qt.callLater(root.refreshContent)
    }

    function deactivate() {
        root.enableShortcuts = false
    }
}