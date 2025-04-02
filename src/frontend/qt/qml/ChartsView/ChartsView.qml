pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../js/TextTransform.js" as TextTransform
import "../StyleOverride"


Item {
    id: root
    property string hostId: ""
    property int groupHeight: 300
    property int chartHeight: 120
    property int columnSpacing: Theme.spacingNormal
    property bool enableShortcuts: false
    property var _categories: {}
    property bool _showEmptyCategories: true

    signal refreshRequested()

    Component.onCompleted: {
        root._categories = []
        if (root.hostId !== "") {
            root._categories =  LK.metrics.getCategories(root.hostId)
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

    ScrollView {
        id: rootScrollView
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
                    Layout.minimumHeight: root.groupHeight
                    Layout.maximumHeight: root.groupHeight
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

                    Column {
                        id: chartColumn
                        width: parent.width
                        spacing: 0

                        Repeater {
                            model: LK.metrics.getCategoryMonitorIds(root.hostId, groupBox.categoryName)

                            LineChart {
                                id: chart
                                required property var modelData
                                property string monitorId: modelData
                                property int invocationId: -1

                                width: parent.width / 2
                                height: root.chartHeight

                                Connections {
                                    target: root

                                    function onRefreshRequested() {
                                        chart.invocationId = LK.metrics.refreshCharts(root.hostId, monitorId)
                                    }
                                }

                                Connections {
                                    target: LK.metrics

                                    function onDataReceived(invocationId, chartDataJson) {
                                        if (invocationId === chart.invocationId) {
                                            let chartDatas = JSON.parse(chartDataJson)
                                            if (chart.monitorId in chartDatas) {
                                                let newValues = chartDatas[chart.monitorId]
                                                    .map(item => { return {"t": item.time * 1000, "y": item.value}})
                                                chart.setData(newValues)
                                                console.log("chart data: " + JSON.stringify(chartDatas))
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
    }

    function refreshContent() {
        root.refreshRequested()
    }

    function activate() {
        // TODO: this tab gets activated initially even if main view is the active view so this might impact performance.
        root.enableShortcuts = true
        root.refreshContent()
    }

    function deactivate() {
        root.enableShortcuts = false
    }
}