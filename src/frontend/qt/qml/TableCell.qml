import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property bool firstItem: false
    property bool selected: false
    implicitHeight: 40
    implicitWidth: parent.width

    signal clicked()

    Rectangle {
        id: rounded
        anchors.fill: parent
        radius: parent.firstItem ? 9 : 0
        color: getBackgroundColor(root.selected)

        MouseArea {
            anchors.fill: parent
            onClicked: root.clicked()
        }
    }

    Rectangle {
        color: getBackgroundColor(root.selected)
        width: rounded.radius
        anchors.top: rounded.top
        anchors.bottom: rounded.bottom
        anchors.right: rounded.right
    }

    function getBackgroundColor(selected) {
        if (selected === true) {
            return Material.primary
        }
        else if (model.row % 2 == 0) {
            return Material.background
        }
        else {
            return Qt.darker(Material.background, 1.10)
        }
    }
}