import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    // NOTE: remember that required properties can cause issues if used with modelData.
    required property string placeholder
    required property string text

    implicitWidth: textComponent.width
    implicitHeight: textComponent.height

    NormalText {
        id: textComponent
        anchors.verticalCenter: parent.verticalCenter
        text: parent.text.length === 0 ? placeholder : parent.text
        font.italic: parent.text.length === 0
        opacity: parent.text.length === 0 ? 0.3 : 1
    }
}