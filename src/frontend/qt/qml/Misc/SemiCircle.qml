import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Shapes 1.15



// Semi-circle, with round end to the left.
Shape {
    id: root
    property int radius: 20
    property alias color: shapePath.fillColor
    width: radius * 2
    height: parent.height

    ShapePath {
        id: shapePath
        strokeColor: "transparent"
        startX: root.width / 2
        startY: 0

        PathArc {
            direction: PathArc.Counterclockwise
            useLargeArc: true
            relativeX: 0
            relativeY: root.height
            radiusX: root.width / 2
            radiusY: root.height / 2
        }
    }
} 