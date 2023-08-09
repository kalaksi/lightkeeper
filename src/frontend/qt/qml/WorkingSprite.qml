import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import "Text"


Item {
    id: root
    property real scale: 1.0
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