import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls.Material 2.15

Item {
    required property string placeholder
    required property string text
    anchors.fill: parent

    NormalText {
        anchors.verticalCenter: parent.verticalCenter
        text: parent.text.length === 0 ? placeholder : parent.text
        font.italic: parent.text.length === 0
        opacity: parent.text.length === 0 ? 0.3 : 1
    }
}