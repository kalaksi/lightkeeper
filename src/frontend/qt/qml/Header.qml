import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    required property string text
    property string color: "#555555"

    property bool _maximized: false

    implicitWidth: parent.width
    implicitHeight: 35

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
            font.pointSize: 16
        }
    }

    ImageButton {
        id: maximizeButton
        anchors.right: closeButton.left
        anchors.rightMargin: 5
        anchors.verticalCenter: parent.verticalCenter

        imageSource: "qrc:/main/images/button/maximize"
        onClicked: {
            root.maximizeClicked()
            root._maximized = true
        }

        visible: root._maximized === false
    }

    ImageButton {
        id: minimizeButton
        anchors.right: closeButton.left
        anchors.rightMargin: 5
        anchors.verticalCenter: parent.verticalCenter

        imageSource: "qrc:/main/images/button/minimize"
        onClicked: {
            root.minimizeClicked()
            root._maximized = false
        }

        visible: root._maximized === true
    }

    ImageButton {
        id: closeButton
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: 10

        imageSource: "qrc:/main/images/button/close"
        imageRelativeWidth: 0.5
        imageRelativeHeight: 0.8
        onClicked: root.closeClicked()
    }
}