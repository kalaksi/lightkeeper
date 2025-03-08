import QtQuick
import Qt5Compat.GraphicalEffects


Item {
    id: root
    property bool antialiasing: false
    property alias source: image.source
    property alias color: overlay.color
    property alias sourceSize: image.sourceSize

    Image {
        id: image
        anchors.fill: parent
        source: root.source
        antialiasing: root.antialiasing
    }

    ColorOverlay {
        id: overlay
        anchors.fill: image
        source: image
        antialiasing: root.antialiasing
    }
}