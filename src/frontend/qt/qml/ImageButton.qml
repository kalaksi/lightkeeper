import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    required property string imageSource
    property real imageRelativeWidth: 0.9
    property real imageRelativeHeight: 0.9
    property string color: Material.foreground
    property string tooltip: ""

    width: 0.8 * parent.height
    height: width

    signal clicked()

    Button {
        flat: true
        anchors.fill: parent
        anchors.centerIn: parent

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