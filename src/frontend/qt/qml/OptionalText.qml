import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    id: container
    required property string placeholder
    required property string text
    property string color: "#aaaaaa"
    anchors.fill: parent

    Text {
        text: parent.text.length === 0 ? placeholder : parent.text
        color: parent.text.length === 0 ? container.color : ""
        font.italic: parent.text.length === 0
        anchors.verticalCenter: parent.verticalCenter
    }
}