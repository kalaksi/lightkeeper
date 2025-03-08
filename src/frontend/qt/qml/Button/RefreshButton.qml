import QtQuick
import QtQuick.Controls

import Theme

Item {
    id: root
    property bool spinOnClick: false
    property bool spinning: false
    property string imageSource: "qrc:/main/images/button/refresh"
    property real imageRelativeWidth: 0.8
    property real imageRelativeHeight: 0.8
    property bool flatButton: true
    property real size: 0.8 * parent.height
    property int iconWidth: Math.floor(imageRelativeWidth * size)
    property int iconHeight: Math.floor(imageRelativeHeight * size)

    width: root.size
    height: root.size

    signal clicked()

    Button {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent

        ToolTip.visible: hovered
        ToolTip.delay: Theme.tooltipDelay
        ToolTip.text: "Refresh"

        onClicked: {
            if (root.spinOnClick) {
                root.spinning = true
            }

            root.clicked()
        }

        Image {
            id: image
            anchors.centerIn: parent
            source: root.imageSource
            width: root.iconWidth
            height: root.iconHeight

            NumberAnimation on rotation {
                from: 0
                to: 360
                loops: Animation.Infinite
                duration: 1000
                running: root.spinning
                alwaysRunToEnd: false

                onStopped: {
                    image.rotation = 0
                    readyAnimation.start()
                }
            }

        }

        Image {
            id: readyAnimationImage
            visible: false
            anchors.centerIn: parent
            source: root.imageSource
            width: root.iconWidth
            height: root.iconHeight
            z: 10
        }

        ParallelAnimation {
            id: readyAnimation
            onStarted: {
                readyAnimationImage.visible = true
            }

            onStopped: {
                readyAnimationImage.visible = false
                readyAnimationImage.scale = 1.0
                readyAnimationImage.opacity = 1.0
            }

            PropertyAnimation {
                target: readyAnimationImage
                property: "opacity"
                to: 0.2
                duration: Theme.animationDuration
            }

            PropertyAnimation {
                target: readyAnimationImage
                property: "scale"
                to: 2.0
                duration: Theme.animationDuration
            }
        }
    }
}