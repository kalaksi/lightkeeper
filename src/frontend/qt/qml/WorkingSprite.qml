/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import "Text"


/// Don't put directly under Layout-components. Wrap inside an Item then.
Item {
    id: root
    property real scale: 1.5
    property string text: ""
    /// When false, sprite fades out. Used by parent to trigger fade before hiding.
    property bool show: true

    opacity: 1
    visible: opacity > 0
    anchors.centerIn: parent
    anchors.verticalCenterOffset: -0.1 * parent.height

    onShowChanged: {
        if (root.show) {
            root.opacity = 1
        }
        else {
            fadeOut.start()
        }
    }

    NumberAnimation {
        id: fadeOut
        target: root
        property: "opacity"
        to: 0
        duration: 120
    }

    Column {
        anchors.fill: parent
        spacing: 10

        AnimatedSprite {
            id: sprite
            anchors.horizontalCenter: parent.horizontalCenter
            source: "qrc:/main/images/animations/working"
            frameWidth: 22
            frameHeight: 22
            frameCount: 15
            frameDuration: 60
            scale: root.scale
        }

        NormalText {
            id: textContent
            anchors.horizontalCenter: parent.horizontalCenter
            visible: root.text !== ""
            text: root.text
        }
    }
}