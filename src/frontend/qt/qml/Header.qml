import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "Text"

Item {
    id: root
    property string text: ""
    property string color: "#555555"
    property bool showRefreshButton: true
    property bool showMinimizeButton: true
    property bool showMaximizeButton: true
    property bool showCloseButton: true
    property bool showOpenInWindowButton: false

    property bool _maximized: false

    implicitWidth: parent.width
    implicitHeight: 30

    signal refreshClicked()
    signal openInWindowClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal closeClicked()

    Rectangle {
        color: root.color
        anchors.fill: parent

        NormalText {
            anchors.verticalCenter: parent.verticalCenter
            leftPadding: 10
            text: root.text
            font.pointSize: 12
        }
    }

    Row {
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.right: parent.right
        anchors.rightMargin: 5
        spacing: 5

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter
            imageSource: "qrc:/main/images/button/refresh"
            imageRelativeWidth: 0.8
            imageRelativeHeight: 0.8
            tooltip: "Refresh monitors"
            onClicked: root.refreshClicked()
            visible: root.showRefreshButton
        }

        ImageButton {
            anchors.rightMargin: 5
            anchors.verticalCenter: parent.verticalCenter
            imageSource: "qrc:/main/images/button/window-new"
            tooltip: "Open in new window"
            onClicked: root.openInWindowClicked()
            visible: root.showOpenInWindowButton
        }

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter
            imageSource: "qrc:/main/images/button/maximize"
            tooltip: "Maximize"
            onClicked: {
                root.maximizeClicked()
                root._maximized = true
            }
            visible: root.showMaximizeButton && !root._maximized
        }

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter
            imageSource: "qrc:/main/images/button/minimize"
            tooltip: "Minimize"
            onClicked: {
                root.minimizeClicked()
                root._maximized = false
            }
            visible: root.showMinimizeButton && root._maximized
        }

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter
            imageSource: "qrc:/main/images/button/close"
            color: Qt.darker(Material.foreground, 1.10)
            imageRelativeWidth: 0.5
            imageRelativeHeight: 0.8
            tooltip: "Close"
            onClicked: root.closeClicked()
            visible: root.showCloseButton
        }
    }
}