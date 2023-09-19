import QtQuick 2.15
import QtQuick.Controls 2.15

Rectangle {
    id: root
    property int borderBottom: 0
    property int borderTop: 0
    property int borderLeft: 0
    property int borderRight: 0
    property string borderColor: palette.base
    property alias backgroundColor: background.color

    color: borderColor
    width: parent.width
    height: parent.height

    Rectangle {
        id: background
        anchors.fill: parent
        anchors.leftMargin: root.borderLeft
        anchors.rightMargin: root.borderRight
        anchors.topMargin: root.borderTop
        anchors.bottomMargin: root.borderBottom
    }
}