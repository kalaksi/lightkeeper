import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property bool spinOnClick: false
    property bool spinning: false
    property string imageSource: "qrc:/main/images/button/refresh"
    property real imageRelativeWidth: 0.8
    property real imageRelativeHeight: 0.8
    property bool flatButton: true

    width: 0.8 * parent.height
    height: width

    signal clicked()

    Button {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent

        ToolTip.visible: hovered
        ToolTip.delay: Theme.tooltip_delay()
        ToolTip.text: "Refresh"

        onClicked: {
            if (root.spinOnClick) {
                root.spinning = true
            }

            root.clicked()
        }

        Image {
            anchors.centerIn: parent
            source: root.imageSource
            width: root.imageRelativeWidth * root.width
            height: root.imageRelativeHeight * root.height

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: "transparent"
                antialiasing: true
            }

            NumberAnimation on rotation {
                id: spinningAnimation
                from: 0
                to: 360
                loops: Animation.Infinite
                duration: 1000
                running: root.spinning
                alwaysRunToEnd: true
            }
        }
    }
}