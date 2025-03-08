import QtQuick
import QtQuick.Shapes

Shape {
    id: root
    property alias color: shape.fillColor

    ShapePath {
        id: shape
        fillColor: "#ffffff"
        strokeWidth: 0
        strokeColor: "transparent"
        startX: 0
        startY: 0

        // Top edge.
        PathLine { x: 150; y: 0 }
        // Right slant.
        PathLine { x: 200; y: 100 }
        // Bottom edge.
        PathLine { x: 0; y: 100 }
        // Close the shape.
        PathLine { x: 0; y: 0 }
    }
}