import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import ".."
import "../Text"


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

    Component.onCompleted: {
        root._categories = []
        refresh()
    }

    Connections {
        target: LK.charts

        function onDataReceived(invocationId, chartDataJson) {
            // TODO
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
        text: "Connecting..."
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
            }
        }
    }
    function refresh() {
        if (root.hostId !== "") {
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