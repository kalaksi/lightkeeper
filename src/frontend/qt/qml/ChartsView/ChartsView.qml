import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../DetailsView"


Item {
    id: root
    property string hostId: ""
    property int columnMinimumWidth: Theme.groupboxMinWidth
    property int columnMaximumWidth: Theme.groupboxMaxWidth
    property int columnMinimumHeight: 450
    property int columnMaximumHeight: 450
    property int columnSpacing: Theme.spacingNormal
    property bool enableShortcuts: false
    property var _categories: {}
    property bool _showEmptyCategories: true

    Component.onCompleted: {
        root._categories = []
        root.refresh()
    }

    Connections {
        target: LK.charts

        function onDataReceived(invocationId, chartDataJson) {
            if (hostId === root.hostId) {
                root.refresh()
            }
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
            columns: Math.floor(parent.width / root.columnMinimumWidth)
            columnSpacing: root.columnSpacing

            Repeater {
                model: root._categories

                GroupBox {
                    id: groupBox
                    property bool blocked: true
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

                    // Custom label provides more flexibility.
                    label: GroupBoxLabel {
                        id: groupBoxLabel
                        anchors.left: groupBox.left
                        anchors.right: groupBox.right

                        text: TextTransform.capitalize(modelData)
                        icon: Theme.categoryIcon(modelData)
                        color: Theme.categoryColor(modelData)
                        onRefreshClicked: function() {
                            // TODO
                        }
                    }
                }
            }
        }
    }

    function refresh() {
        if (root.hostId !== "") {
            root._categories =  LK.hosts.getCategories(root.hostId, !root._showEmptyCategories)
                                        .map(category_qv => category_qv.toString())

        }
    }

    function refreshContent() {
        for (const category of root._categories) {
            for (const monitorId of LK.hosts.getCategoryMonitorIds(root.hostId, category)) {
                LK.charts.refreshCharts(hostId, monitorId)
            }
        }
    }

    function activate() {
        root.enableShortcuts = true
    }

    function deactivate() {
        root.enableShortcuts = false
    }

    function close() {
        // Do nothing.
    }
}