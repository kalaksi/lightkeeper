import QtQuick 2.15

import "Text"
import "js/Utils.js" as Utils


Rectangle {
    id: root
    required property string text
    required property string criticality
    property int contentPadding: 10
    property int fadeDuration: 200
    property int showDuration: 5000
    property int maximumWidth: 600

    visible: getText() !== ""
    width: Utils.clamp(textContent.implicitWidth + iconBackground.width + root.contentPadding * 3, root.maximumWidth / 3, root.maximumWidth)
    implicitHeight: Math.max(textContent.implicitHeight, image.height) + root.contentPadding
    radius: 5
    opacity: 0.0
    color: Theme.backgroundColor
    border.width: 1
    border.color: "#50FFFFFF"
    // Alternative way to get some matching color to border:
    // border.color: Qt.darker(Qt.lighter(getColor(), 1.5), 2.0)

    Rectangle {
        id: iconBackground
        anchors.left: parent.left
        anchors.leftMargin: root.border.width
        // "- root.border.width * 2" might be wrong, but otherwise it's not centered.
        width: image.width + root.contentPadding * 2 + iconBackgroundCutoff.width - root.border.width * 2
        height: row.height - root.border.width * 2
        anchors.verticalCenter: parent.verticalCenter
        color: getColor()
        radius: 5
    }

    // Cut the rounding on right side
    Rectangle {
        id: iconBackgroundCutoff
        anchors.right: iconBackground.right
        anchors.rightMargin: -root.border.width
        width: parent.radius
        height: iconBackground.height
        anchors.verticalCenter: parent.verticalCenter
        color: root.color
    }

    Row {
        id: row
        padding: root.contentPadding
        spacing: iconBackgroundCutoff.width + root.contentPadding * 2
        anchors.fill: parent
        anchors.centerIn: parent

        Image {
            id: image
            anchors.verticalCenter: parent.verticalCenter
            antialiasing: true
            source: Theme.icon_for_criticality(root.criticality)
            width: 32
            height: 32
        }

        NormalText {
            id: textContent
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            wrapMode: Text.Wrap
            width: row.width - iconBackground.width - root.contentPadding * 2
            height: row.height - root.contentPadding * 2
            // TODO: handle overly long messages better
            clip: true
        }
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

    // When hovering, set opacity to 1.0 and stop the animation.
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onEntered: {
            animation.stop()
            root.opacity = 1.0
        }
        onExited: {
            animation.start()
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
            return "#F25560"
        }
        else if (root.criticality === "Error") {
            return "#FF6065"
        }
        else if (root.criticality === "Warning") {
            return "#FFC734"
        }
        else if (root.criticality === "Info") {
            return Theme.backgroundColorDark
        }
        else if (root.criticality === "Normal") {
            return Theme.backgroundColorDark
        }
        else if (root.criticality === "NoData") {
            return "#FFC734"
        }
    }
}