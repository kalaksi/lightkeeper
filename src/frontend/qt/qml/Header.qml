import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property string text: ""
    property string color: "#555555"
    property bool showMinimizeButton: true
    property bool showMaximizeButton: true
    property bool showCloseButton: true
    property bool showOpenInWindowButton: false

    property bool _maximized: false

    implicitWidth: parent.width
    implicitHeight: 30

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
            anchors.rightMargin: 5
            anchors.verticalCenter: parent.verticalCenter

            imageSource: "qrc:/main/images/button/window-new"
            onClicked: root.openInWindowClicked()
            visible: root.showOpenInWindowButton
        }

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter

            imageSource: "qrc:/main/images/button/maximize"
            onClicked: {
                root.maximizeClicked()
                root._maximized = true
            }

            visible: root.showMaximizeButton && !root._maximized
        }

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter

            imageSource: "qrc:/main/images/button/minimize"
            onClicked: {
                root.minimizeClicked()
                root._maximized = false
            }

            visible: root.showMinimizeButton && root._maximized
        }

        ImageButton {
            anchors.verticalCenter: parent.verticalCenter

            imageSource: "qrc:/main/images/button/close"
            imageRelativeWidth: 0.5
            imageRelativeHeight: 0.8
            onClicked: root.closeClicked()

            visible: root.showCloseButton
        }

    }
}