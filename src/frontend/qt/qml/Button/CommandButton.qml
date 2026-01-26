/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick


Item {
    id: root
    property string buttonId: ""
    property int size: 0.8 * parent.height
    property alias tooltip: imageButton.tooltip
    property alias imageSource: imageButton.imageSource
    property alias roundButton: imageButton.roundButton
    property alias hoverEnabled: imageButton.hoverEnabled
    property int progressPercent: 100
    property real circlingAngle: 0.0

    onProgressPercentChanged: {
        progressCanvas.requestPaint()
    }

    width: root.size
    height: root.size

    signal clicked()

    ImageButton {
        id: imageButton
        anchors.fill: parent
        flatButton: false
        roundButton: true
        onClicked: () => root.clicked()
    }

    // progress animation that blocks the button until timeout timer is finished or finish() called.
    Canvas {
        id: progressCanvas
        visible: root.isInProgress()
        anchors.fill: parent
        onPaint: {
            let context = getContext("2d")
            let radius = width / 2
            context.clearRect(0, 0, width, height)
            context.save()

            // A pie that gets smaller clockwise.
            context.translate(radius, radius)
            context.rotate(-Math.PI / 2)
            context.beginPath()
            context.arc(0, 0, radius, 0, 2 * Math.PI * root.progressPercent * 0.01 + 0.001, true)
            context.lineTo(0, 0)
            context.closePath()
            context.fillStyle = "#60FFFFFF"
            context.fill()

            context.restore()
        }
    }

    Canvas {
        id: circlingCanvas 
        visible: root.isInProgress()
        anchors.fill: parent
        onPaint: {
            let context = getContext("2d")
            let lineWidth = 3
            let radius = width / 2
            context.clearRect(0, 0, width, height)
            context.save()

            // Circling arcs around the round button.
            context.translate(radius, radius)
            context.rotate(-Math.PI / 2 + root.circlingAngle)
            context.strokeStyle = "#C0FFFFFF"
            context.lineWidth = lineWidth
            context.beginPath()
            context.arc(0, 0, radius - lineWidth / 2.0, 0, Math.PI / 2.0)
            context.stroke()
            context.rotate(Math.PI)
            context.beginPath()
            context.arc(0, 0, radius - lineWidth / 2.0, 0, Math.PI / 2.0)
            context.stroke()

            context.restore()
        }
    }

    // Block mouse events from reaching the button.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        propagateComposedEvents: false
        visible: progressCanvas.visible
    }

    Timer {
        id: circlingTimer
        interval: 50
        running: root.isInProgress()
        repeat: true

        onTriggered: {
            root.circlingAngle += Math.PI * 2.0 / (2000.0 / interval)

            if (root.circlingAngle >= Math.PI * 2.0) {
                root.circlingAngle = 0.0
            }

            circlingCanvas.requestPaint()
        }
    }

    function isInProgress() {
        return root.progressPercent >= 0 && root.progressPercent < 100
    }
}