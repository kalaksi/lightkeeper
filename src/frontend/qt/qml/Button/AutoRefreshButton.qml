import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0

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

    enabled: true
    opacity: enabled ? 1.0 : 0.5
    width: root.size * 2.0
    height: root.size

    signal clicked()



    // Auto-refresh timer. Triggered for every percentage change in progress.
    Timer {
        id: autoRefreshTimer
        // Value for 1 h.
        interval: 36000
        running: false
        repeat: true

        onTriggered: {
            refreshTimerCanvas.progressPercent += 1

            if (refreshTimerCanvas.progressPercent >= 100) {
                refreshTimerCanvas.progressPercent = 0
                root.clicked()
            }

            refreshTimerCanvas.requestPaint()
        }
    }

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

        Row {
            anchors.centerIn: parent
            height: parent.height
            padding: 4

            Image {
                id: image
                source: root.imageSource
                width: root.iconWidth
                height: root.iconHeight
                anchors.verticalCenter: parent.verticalCenter

                Canvas {
                    id: refreshTimerCanvas
                    property int progressPercent: 0

                    visible: autoRefreshTimer.running
                    anchors.fill: parent

                    onPaint: {
                        let context = getContext("2d")
                        let lineWidth = 2
                        let radius = width / 2
                        context.clearRect(0, 0, width, height)
                        context.save()

                        // A pie that gets smaller clockwise.
                        context.translate(radius, radius)
                        context.rotate(-Math.PI / 2)
                        context.strokeStyle = palette.highlight
                        context.lineWidth = lineWidth
                        context.beginPath()
                        context.arc(0, 0, radius - lineWidth / 2.0, 0, 2 * Math.PI * progressPercent * 0.01 + 0.001, true)
                        context.stroke()

                        context.restore()
                    }
                }

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

            // Highlighting background for the menu part.
            Rectangle {
                id: dropdownBackground
                width: root.iconWidth
                anchors.verticalCenter: parent.verticalCenter
                height: parent.height
                color: "transparent"

                Image {
                    id: dropdownIcon
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/dropdown-menu"
                    width: Math.floor(0.8 * root.iconWidth)
                    height: Math.floor(0.8 * root.iconHeight)

                    MouseArea {
                        anchors.fill: parent
                        // Don't allow clickthrough.
                        preventStealing: true
                        hoverEnabled: true
                        onEntered: {
                            dropdownBackground.color = Theme.highlightColorLight
                        }
                        onExited: {
                            dropdownBackground.color = "transparent"
                        }
                        onClicked: {
                            if (refreshMenu.visible) {
                                refreshMenu.close()
                            }
                            else {
                                refreshMenu.open()
                            }
                        }
                    }

                    Menu {
                        id: refreshMenu 
                        title: "Auto-refresh"
                        visible: false
                        x: parent.x - implicitWidth + parent.width
                        y: parent.y + parent.height

                        function setTimerInterval(intervalSec) {
                            if (intervalSec === 0) {
                                autoRefreshTimer.stop()
                            }
                            else {
                                refreshTimerCanvas.progressPercent = 0
                                refreshTimerCanvas.requestPaint()
                                // Timer is triggered for every percentage change in progress.
                                // In other words, (intervalSec * 1000) / 100
                                autoRefreshTimer.interval = intervalSec * 10
                                autoRefreshTimer.restart()
                            }

                            refreshMenu.close()
                        }

                        // MenuItem doesn't work well with Repeater in older Qt?
                        // TODO: Refactor after upgrading to Qt 6?
                        MenuItem {
                            text: "Auto-refresh off"
                            onClicked: refreshMenu.setTimerInterval(0)
                        }
                        MenuItem {
                            text: "Every 5 m"
                            onClicked: refreshMenu.setTimerInterval(300)
                        }

                        MenuItem {
                            text: "Every 15 m"
                            onClicked: refreshMenu.setTimerInterval(900)
                        }

                        MenuItem {
                            text: "Every 1 h"
                            onClicked: refreshMenu.setTimerInterval(3600)
                        }

                        MenuItem {
                            text: "Every 3 h"
                            onClicked: refreshMenu.setTimerInterval(10800)
                        }

                        MenuItem {
                            text: "Every 12 h"
                            onClicked: refreshMenu.setTimerInterval(43200)
                        }

                        MenuItem {
                            text: "Every 24 h"
                            onClicked: refreshMenu.setTimerInterval(86400)
                        }
                    }
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