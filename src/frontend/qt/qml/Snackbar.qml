import QtQuick 2.15
import QtQuick.Controls.Material 2.15


Rectangle {
    id: root
    required property string text
    required property string criticality
    property int fadeDuration: 200
    property int showDuration: 2000
    property int contentPadding: 20
    property int contentWidth: 500
    property int contentHeight: 50


    visible: getText() !== ""
    implicitWidth: root.contentWidth + root.contentPadding
    implicitHeight: alertMessage.height + root.contentPadding * 2
    radius: 5
    opacity: 0.0
    color: getColor()
    border.width: 1
    border.color: Qt.lighter(getColor(), 1.75)

    AlertMessage {
        id: alertMessage
        text: getText()
        criticality: root.criticality
        width: root.contentWidth
        imageScale: 1.4
        anchors.centerIn: parent
    }

    SequentialAnimation on opacity {
        id: animation

        NumberAnimation {
            to: 0.8
            duration: root.fadeDuration
        }

        PauseAnimation {
            duration: root.showDuration
        }

        NumberAnimation {
            to: 0.0
            duration: root.fadeDuration
        }
    }

    function getText() {
        if (root.text === "") {
            if (root.criticality === "Error") {
                return "Unknown error occurred"
            } 
            else {
                return ""
            }
        }
        else {
            return root.text
        }
    }

    function getColor() {
        if (root.criticality === "Critical") {
            return "#FF8045"
        }
        else if (root.criticality === "Error") {
            return "#FF8045"
        }
        else if (root.criticality === "Warning") {
            return "#FFF0C0"
        }
        else if (root.criticality === "Info") {
            return Qt.darker(Material.background, 1.15)
        }
        else if (root.criticality === "Normal") {
            return Qt.darker(Material.background, 1.15)
        }
        else if (root.criticality === "NoData") {
            return "#FFF0C0"
        }
    }
}