import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

Dialog {
    id: root
    modal: false
    standardButtons: Dialog.Ok

    property var commandResults: []

    AnimatedSprite {
        source: "qrc:/main/images/animations/working"
        frameWidth: 22
        frameHeight: 22
        frameCount: 15
        frameDuration: 50

        anchors.centerIn: parent
    }

    Row {
        anchors.fill: parent

        Repeater {
            model: root.commandResults

            Text {
                text: modelData.message
            }
        }

    }

}