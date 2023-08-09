import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

// TODO: rename to WaveAnimation.
Item {
    id: root
    property string color: "#555555"
    property real radius: (parent.width < parent.height ? parent.width : parent.height) * 0.5
    property int duration: 500

    Rectangle {
        id: rectangle
        anchors.centerIn: parent
        radius: root.radius
        color: root.color

        opacity: 0.7
        width: radius * 1.5
        height: width

        NumberAnimation on radius {
            to: root.radius * 2.5
            duration: root.duration
        }

        NumberAnimation on opacity {
            to: 0.0
            duration: root.duration
        }
    }

    states: [
        // Set visibility to false when completely transparent to disable unneeded components.
        // Not sure if this is needed or already done automatically.
        State {
            name: "hideWhenTransparent"
            when: rectangle.opacity < 0.01

            PropertyChanges {
                target: root
                visible: false
            }
        }
    ]
}