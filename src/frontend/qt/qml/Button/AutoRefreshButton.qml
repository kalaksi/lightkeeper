/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Lightkeeper 1.0

import "../StyleOverride"

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
    /// In seconds. 0 disables auto-refresh.
    property int autoRefreshInterval: 0
    /// Seconds until the next core auto-refresh trigger.
    property int nextAutoRefreshIn: 0
    readonly property string autoRefreshLabel: {
        switch (autoRefreshInterval) {
            case 0: return "auto-refresh off"
            case 300: return "every 5 m"
            case 900: return "every 15 m"
            case 3600: return "every 1 h"
            case 10800: return "every 3 h"
            case 21600: return "every 6 h"
            case 43200: return "every 12 h"
            case 86400: return "every 24 h"
            default: return "every " + autoRefreshInterval + " s"
        }
    }

    enabled: true
    opacity: enabled ? 1.0 : 0.5
    width: root.size * 2.0
    height: root.size

    signal clicked()
    signal intervalSelected(int intervalSec)

    Button {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent

        ToolTip.visible: hovered
        ToolTip.delay: Theme.tooltipDelay
        ToolTip.text: "Refresh (" + root.autoRefreshLabel + ")"

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

            Item {
                width: root.iconWidth
                height: root.iconHeight
                anchors.verticalCenter: parent.verticalCenter

                Image {
                    id: image
                    anchors.fill: parent
                    source: root.imageSource

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

                // Keep the progress ring still while the refresh icon spins.
                Canvas {
                    id: refreshTimerCanvas
                    readonly property int progressPercent: root.autoRefreshInterval > 0
                        ? Math.round(100 * (1 - root.nextAutoRefreshIn / root.autoRefreshInterval))
                        : 0

                    visible: root.autoRefreshInterval > 0
                    anchors.fill: parent

                    onProgressPercentChanged: requestPaint()

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
                        width: dropdownBackground.width
                        height: dropdownBackground.height
                        anchors.centerIn: parent
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
                            root.autoRefreshInterval = intervalSec
                            root.intervalSelected(intervalSec)
                            refreshMenu.close()
                        }

                        MenuItem {
                            text: "Auto-refresh off"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 0
                            onClicked: refreshMenu.setTimerInterval(0)
                        }
                        MenuItem {
                            text: "Every 5 m"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 300
                            onClicked: refreshMenu.setTimerInterval(300)
                        }

                        MenuItem {
                            text: "Every 15 m"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 900
                            onClicked: refreshMenu.setTimerInterval(900)
                        }

                        MenuItem {
                            text: "Every 1 h"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 3600
                            onClicked: refreshMenu.setTimerInterval(3600)
                        }

                        MenuItem {
                            text: "Every 3 h"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 10800
                            onClicked: refreshMenu.setTimerInterval(10800)
                        }

                        MenuItem {
                            text: "Every 6 h"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 21600
                            onClicked: refreshMenu.setTimerInterval(21600)
                        }

                        MenuItem {
                            text: "Every 12 h"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 43200
                            onClicked: refreshMenu.setTimerInterval(43200)
                        }

                        MenuItem {
                            text: "Every 24 h"
                            checkable: true
                            autoExclusive: true
                            checked: root.autoRefreshInterval === 86400
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
