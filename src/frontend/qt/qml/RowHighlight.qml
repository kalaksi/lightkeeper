import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

Rectangle {
    id: root
    color: Theme.background_color()
    property alias containsMouse: mouseArea.containsMouse

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        preventStealing: true

        onEntered: {
            root.color = Theme.highlight_color()
        }

        onExited: {
            root.color = Theme.background_color()
        }

        // Child components get put here.
        Item {
            id: contentItem
            anchors.fill: parent
        }
    }
}
