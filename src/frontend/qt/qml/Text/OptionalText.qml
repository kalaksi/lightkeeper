import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    // NOTE: Required properties can cause issues if used with modelData, like errors such as:
    // "ReferenceError: modelData is not defined"
    // That's why these are not required properties anymore:
    property string placeholder: ""
    property string text: ""

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