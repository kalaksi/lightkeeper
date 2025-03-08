import QtQuick

import "Text"


/// Don't put directly under Layout-components. Wrap inside an Item then.
Item {
    id: root
    property real scale: 1.5
    property string text: ""

    anchors.centerIn: parent
    anchors.verticalCenterOffset: -0.1 * parent.height

    Column {
        anchors.fill: parent
        spacing: 10

        AnimatedSprite {
            id: sprite
            anchors.horizontalCenter: parent.horizontalCenter
            source: "qrc:/main/images/animations/working"
            frameWidth: 22
            frameHeight: 22
            frameCount: 15
            frameDuration: 60
            scale: root.scale
        }

        NormalText {
            id: textContent
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.text !== ""
            text: root.text
        }
    }
}