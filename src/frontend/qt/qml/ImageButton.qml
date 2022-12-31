import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property string imageSource: ""
    property real imageRelativeWidth: 0.9
    property real imageRelativeHeight: 0.9
    property string color: "transparent"
    property string tooltip: ""
    property bool roundButton: false
    property bool flatButton: true

    width: 0.8 * parent.height
    height: width

    signal clicked()

    Button {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent
        visible: roundButton === false
        onClicked: root.clicked()

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: 800
        ToolTip.text: root.tooltip

        Image {
            anchors.centerIn: parent
            source: root.imageSource
            width: root.imageRelativeWidth * root.width
            height: root.imageRelativeHeight * root.height

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: root.color
                antialiasing: true
            }
        }
    }

    RoundButton {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent
        visible: roundButton === true
        onClicked: root.clicked()

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: 800
        ToolTip.text: root.tooltip

        Image {
            anchors.centerIn: parent
            source: root.imageSource
            width: root.imageRelativeWidth * root.width
            height: root.imageRelativeHeight * root.height

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: root.color
                antialiasing: true
            }
        }
    }
}