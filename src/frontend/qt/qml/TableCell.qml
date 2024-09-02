import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    id: root
    default property alias contentItem: contentItem.data
    property bool useRounding: true
    property bool firstItem: false
    property bool selected: false
    property int padding: 0
    implicitHeight: 40
    implicitWidth: parent.width

    signal clicked()

    // Stylish rounded cell for first item.
    Rectangle {
        id: rounded
        anchors.fill: parent
        radius: root.useRounding && parent.firstItem ? 9 : 0
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

    Item {
        id: contentItem
        height: parent.height
        width: parent.width - padding
        anchors.centerIn: parent
    }

    // It seems that at least Qt 6.6 now has native support for alternating row colors.
    // TODO: use the native support when it's available.
    function getBackgroundColor(selected) {
        if (selected === true) {
            return Qt.darker(Theme.highlightColor)
        }
        else if (model.row % 2 == 0) {
            return palette.alternateBase
        }
        else {
            return palette.base
        }
    }
}