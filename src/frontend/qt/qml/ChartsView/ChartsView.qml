pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import ChartJs 1.0

import Theme

import ".."
import "../js/TextTransform.js" as TextTransform


Item {
    id: root
    property string hostId: ""
    property int columnMinimumWidth: rootScrollView.availableWidth / 2
    property int columnMaximumWidth: rootScrollView.availableWidth
    property int columnMinimumHeight: 300
    property int columnMaximumHeight: 300
    property int chartHeight: 100
    property int columnSpacing: Theme.spacingNormal
    property bool enableShortcuts: false
    property var _categories: {}
    property bool _showEmptyCategories: true

    Component.onCompleted: {
        root._categories = []
        if (root.hostId !== "") {
            root._categories =  LK.metrics.getCategories(root.hostId)
        }

        LK.metrics.startService()

        root.refreshContent()
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
            columns: Math.floor(parent.width / root.columnMinimumWidth)
            columnSpacing: root.columnSpacing

            Repeater {
                model: root._categories

                GroupBox {
                    id: groupBox
                    required property var modelData
                    property alias categoryName: groupBox.modelData
                    property var _invocationIdToButton: {}

                    leftPadding: Theme.spacingTight
                    rightPadding: Theme.spacingTight
                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.preferredWidth: root.columnMinimumWidth +
                                           (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) -
                                           root.columnSpacing
                    Layout.minimumHeight: root.columnMinimumHeight
                    Layout.maximumHeight: root.columnMaximumHeight
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

                    ColumnLayout {
                        id: column
                        width: parent.width
                        spacing: 0

                        Chart {
                            id: chart
                            visible: false
                            width: root.columnMinimumWidth
                            height: root.chartHeight
                            chartType: "line"

                            chartData: {
                                return {
                                    datasets: [{
                                        label: "Filled",
                                        fill: true,
                                        backgroundColor: "rgba(192,222,255,0.3)",
                                        borderColor: "rgba(255,255,255,1.0)",
                                        borderWidth: 2,
                                        // pointRadius: 2,
                                        data: [],
                                    }]
                                }
                            }

                            chartOptions: {
                                return {
                                    maintainAspectRatio: false,
                                    responsive: true,
                                    title: {
                                        display: true,
                                        text: groupBox.categoryName,
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
                                            display: false,
                                            type: "time",
                                            time: {
                                                // Unix timestamp in ms.
                                                parser: "x"
                                            },
                                            scaleLabel: {
                                                display: true,
                                                // labelString: "Time"
                                            },
                                            gridLines: {
                                                display: false,
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
                                                labelString: "%",
                                                fontColor: Theme.textColor
                                            },
                                            gridLines: {
                                                display: true,
                                                color: "rgba(255,255,255,0.1)"
                                            },
                                            ticks: {
                                                min: 0.0,
                                                max: 100.0,
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

                            Connections {
                                target: LK.metrics

                                function onDataReceived(invocationId, chartDataJson) {
                                    if (hostId === root.hostId) {
                                        let chartDatas = JSON.parse(chartDataJson)
                                        // console.log("ChartsView.onDataReceived", invocationId, chartDataJson)
                                        for (const monitorId in chartDatas) {
                                            if (monitorId === "ram") {
                                                let newValues = chartDatas[monitorId].map(item => { return {"t": item.time * 1000, "y": item.value}})
                                                chart.chartData.datasets[0].data = newValues
                                                chart.visible = true
                                                chart.animateToNewData()
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
        for (const category of root._categories) {
            for (const monitorId of LK.hosts.getCategoryMonitorIds(root.hostId, category)) {
                LK.metrics.refreshCharts(hostId, monitorId)
            }
        }
    }

    function activate() {
        root.enableShortcuts = true
    }

    function deactivate() {
        root.enableShortcuts = false
    }
}