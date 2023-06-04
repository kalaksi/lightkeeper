import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property bool selected: false
    property bool gradient: false
    property bool firstItem: false
    property bool lastItem: false
    property int radius: 9


    signal clicked()

    // Make sure left side is not rounded
    Rectangle {
        visible: root.lastItem
        color: getBackgroundColor(root.selected)
        width: background.radius
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.left: parent.left
    }

    Rectangle {
        id: background
        visible: root.gradient === false
        anchors.fill: parent
        color: getBackgroundColor(root.selected)
        radius: root.firstItem || root.lastItem ? root.radius : 0

        MouseArea {
            anchors.fill: parent
            onClicked: root.clicked()
        }
    }

    // Backgrounds are also used to clip overflowing text from label on the left.
    // Avoids clip-property on the label itself, since it could cause performance issues if not used sparingly.
    // This also allows more customized style for the clipping.
    Rectangle {
        id: gradientBackground
        visible: root.gradient === true
        x: -parent.width * 0.3
        width: parent.width * 1.3
        height: parent.height
        radius: root.lastItem ? root.radius : 0

        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: "#00000000" }
            GradientStop { position: 0.15; color: getBackgroundColor(root.selected) }
            GradientStop { position: 1.0; color: getBackgroundColor(root.selected) }
        }
    }



    function getBackgroundColor(selected) {
        if (selected === true) {
            return Qt.darker(Theme.category_background_color(), 1.20)
        }
        else {
            return Theme.category_background_color()
        }
    }
}