import QtQuick 2.15
import QtQuick.Controls 2.15

Rectangle {
    id: root
    default property alias contentItem: background.data
    property int border: 0
    property int borderBottom: border > 0 ? border : 0
    property int borderTop: border > 0 ? border : 0
    property int borderLeft: border > 0 ? border : 0
    property int borderRight: border > 0 ? border : 0
    property color borderColor: palette.base
    property alias backgroundColor: background.color

    color: borderColor

    Rectangle {
        id: background
        width: parent.width - borderLeft - borderRight
        height: parent.height - borderTop - borderBottom
        anchors.centerIn: parent
    }
}