import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    property bool firstItem: false

    implicitHeight: 40

    Rectangle {
        id: rounded
        anchors.fill: parent
        radius: parent.firstItem ? 9 : 0
        color: getBackgroundColor()
    }

    Rectangle {
        color: getBackgroundColor()
        width: rounded.radius
        anchors.top: rounded.top
        anchors.bottom: rounded.bottom
        anchors.right: rounded.right
    }

    function getBackgroundColor() {
        return model.row % 2 == 0 ? "#efefef" : "#e5e5e5"
    }
}