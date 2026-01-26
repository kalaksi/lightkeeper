/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Theme


Item {
    id: root
    property bool scrollAnimation: false
    property alias text: text.text
    property int _textOriginalX: 0
    property int _animationTarget: 0

    implicitHeight: text.implicitHeight
    implicitWidth: text.implicitWidth

    onScrollAnimationChanged: {
        if (root.width < text.implicitWidth) {
            if (scrollAnimation) {
                _animationTarget = -text.implicitWidth
                _textOriginalX = text.x
                animation.start()
            } else {
                text.x = root._textOriginalX
                animation.stop()
            }
        }
    }

    Text {
        id: text
        color: Theme.textColor
        font.pointSize: 10
    }

    SmoothedAnimation {
        id: animation 
        target: text
        property: "x"
        to: root._animationTarget
        velocity: 80
        maximumEasingTime: 500

        onFinished: {
            if (root.scrollAnimation) {
                // Just returned to the original position.
                if (root._animationTarget == root._textOriginalX) {
                    root._animationTarget = -text.implicitWidth
                    delayedStart.start()
                } else {
                    // Add some extra so there's a small delay before text is visible again and 
                    // so that any easing effects won't show (bit of a hack to avoid more complicated logic).
                    text.x = root.width + 50
                    root._animationTarget = root._textOriginalX
                    animation.start()
                }
            }
        }
    }

    Timer {
        id: delayedStart
        interval: 2000
        repeat: false

        onTriggered: {
            if (root.scrollAnimation) {
                animation.start()
            }
        }
    }
}