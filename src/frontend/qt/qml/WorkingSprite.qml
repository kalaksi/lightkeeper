import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import "Text"


Item {
    property real scale: 1.0
    property string text: ""

    id: root
    anchors.centerIn: parent

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