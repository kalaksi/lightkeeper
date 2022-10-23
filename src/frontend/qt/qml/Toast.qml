import QtQuick 2.15
import QtQuick.Controls.Material 2.15


Rectangle {
    id: root
    required property string text
    required property string criticality
    property int fadeDuration: 500
    property int showDuration: 6000
    property int contentPadding: 20
    property int contentWidth: 500

    property var colors: {}
    property var icons: {}

    Component.onCompleted: {
        colors = {
            normal: Qt.darker(Material.background),
            info: Qt.darker(Material.background),
            nodata:  "#FFF0C0",
            warning: "#FFF0C0",
            error: "#FF8045",
            critical: "#FF8045",
        }

        icons = {

        }
    }

    width: root.contentWidth + root.contentPadding
    height: alertMessage.height + root.contentPadding * 2
    radius: 5
    opacity: 0.0
    color: colors[root.criticality.toLowerCase()]

    AlertMessage {
        id: alertMessage
        text: root.text
        criticality: root.criticality
        width: root.contentWidth
        y: root.contentPadding
    }

    SequentialAnimation on opacity {
        id: animation

        NumberAnimation {
            to: 0.75
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
}