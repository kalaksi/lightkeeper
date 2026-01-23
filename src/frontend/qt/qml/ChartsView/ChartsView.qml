pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../StyleOverride"


Item {
    id: root
    property string hostId: ""
    property int groupHeight: 300
    property int chartHeight: 120
    property int columnSpacing: Theme.spacingNormal
    property bool enableShortcuts: false
    property var _categories: ({})
    property bool _showEmptyCategories: true

    signal refreshRequested()

    Component.onCompleted: {
        root._categories = []
        if (root.hostId !== "") {
            // TODO: enable more categories later after better support.
            // root._categories = LK.metrics.getCategories(root.hostId)
            root._categories = ["host", "storage"]
        }
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

    Rectangle {
        visible: !LK.config.showCharts
        anchors.centerIn: parent
        anchors.verticalCenterOffset: -parent.height * 0.1
        width: parent.width * 0.6
        height: parent.height * 0.6
        color: Theme.categoryBackgroundColor
        radius: 6

        Column {
            anchors.horizontalCenter: parent.horizontalCenter
            topPadding: 0.2 * parent.height
            height: parent.height
            width: parent.width * 0.8
            spacing: Theme.spacingLoose

            NormalText {
                anchors.horizontalCenter: parent.horizontalCenter
                text: "Charts is still an experimental feature, but can be enabled. Requires restart to take effect."
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                height: 40
                spacing: Theme.spacingLoose

                Switch {
                    id: toggleSwitch
                    checked: false
                    Layout.alignment: Qt.AlignVCenter

                    onCheckedChanged: {
                        LK.config.showCharts = toggleSwitch.checked
                        toggleLabel.text = toggleSwitch.checked ? "Charts are enabled" : "Charts are disabled"
                    }
                }

                Label {
                    id: toggleLabel
                    text: "Charts are disabled"
                    verticalAlignment: Text.AlignVCenter
                }
            }
        }
    }

    ScrollView {
        id: rootScrollView
        visible: LK.config.showCharts
        anchors.fill: parent
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
                    property var _invocationIdToButton: {}

                    leftPadding: Theme.spacingTight
                    rightPadding: Theme.spacingTight
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.minimumHeight: root.groupHeight
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
                        anchors.bottom: parent.bottom
                        spacing: 0
                        columns: 2

                        Repeater {
                            model: LK.metrics.getCategoryMonitorIds(root.hostId, groupBox.categoryName)
                                             .map(monitorId => JSON.parse(LK.hosts.getMonitoringDataJson(root.hostId, monitorId)))
                                             .filter(monitor => monitor.display_options.use_with_charts)

                            Item {
                                id: chart
                                required property var modelData
                                property var monitoringData: modelData
                                property int invocationId: -1
                                // Array of chart data.
                                // Single array for single value charts, multiple arrays for multivalue charts.
                                property var chartDatas: []

                                width: chart.monitoringData.display_options.use_multivalue ? chartColumn.width : chartColumn.width / 2
                                height: chartColumn.height

                                Connections {
                                    target: root

                                    function onRefreshRequested() {
                                        chart.invocationId = LK.metrics.refreshCharts(root.hostId, chart.monitoringData.monitor_id)
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

                                            if (chart.monitoringData.display_options.use_multivalue) {
                                                let labeledData = {};
                                                let validData = chartData.filter(item => item.label !== "")

                                                for (const data of validData) {
                                                    if (labeledData[data.label] === undefined) {
                                                        labeledData[data.label] = [];
                                                    }
                                                    labeledData[data.label].push({"t": data.time * 1000, "y": data.value});
                                                }

                                                chart.chartDatas = Object.keys(labeledData).map(label => {
                                                    return {
                                                        label: chart.monitoringData.display_options.display_text + ": " + label,
                                                        data: labeledData[label]
                                                    }
                                                });
                                            }
                                            else {
                                                let newValues = chartData.map(item => { return {"t": item.time * 1000, "y": item.value} })
                                                chart.chartDatas = [{
                                                    label: chart.monitoringData.display_options.display_text,
                                                    data: newValues
                                                }]
                                            }

                                            // console.log("Chart data for " + chart.monitoringData.display_options.display_text + ":" + JSON.stringify(chart.chartDatas))
                                        }
                                    }
                                }

                                Grid {
                                    id: chartGrid
                                    anchors.fill: parent
                                    columns: 2
                                    columnSpacing: Theme.spacingNormal

                                    property int rowCount: Math.ceil(chart.chartDatas.length / 2)
                                    property real chartHeight: rowCount > 0 ? parent.height / rowCount : root.chartHeight

                                    Repeater {
                                        model: chart.chartDatas

                                        LineChart {
                                            required property var modelData
                                            required property int index

                                            Component.onCompleted: {
                                                // Workaround, couldn't get data to update properly otherwise.
                                                setData(modelData.data)
                                            }

                                            chartData: modelData.data
                                            title: modelData.label
                                            width: chartColumn.width / 2.05
                                            height: chartGrid.chartHeight
                                            yLabel: chart.monitoringData.display_options.unit
                                            yMin: chart.monitoringData.display_options.value_min
                                            yMax: chart.monitoringData.display_options.value_max > 0 ?
                                                chart.monitoringData.display_options.value_max : 100
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    function refreshContent() {
        console.log("Refreshing charts for host: " + root.hostId)
        root.refreshRequested()
    }

    function activate() {
        // TODO: this tab gets activated initially for a short while even if main view is the active view so this might impact performance.
        root.enableShortcuts = true
        root.refreshContent()
    }

    function deactivate() {
        root.enableShortcuts = false
    }
}