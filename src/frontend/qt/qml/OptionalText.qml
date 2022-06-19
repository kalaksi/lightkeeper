import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    id: container
    required property string placeholder
    required property string textContent
    property string color: "#aaaaaa"

    Text {
        text: textContent.length === 0 ? placeholder : textContent
        color: textContent.length === 0 ? container.color : ""
        font.italic: textContent.length === 0
    }
}