/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Theme

Rectangle {
    id: root
    property alias containsMouse: mouseArea.containsMouse
    property bool selected: false

    color: Theme.backgroundColor
    radius: 4

    signal clicked

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        preventStealing: true

        onEntered: {
            if (!root.selected) {
                root.color = Theme.highlightColor
            }
        }

        onExited: {
            if (!root.selected) {
                root.color = Theme.backgroundColor
            }
        }

        onClicked: {
            root.clicked()
            root.selected = !root.selected

            if (root.selected) {
                root.color = Theme.highlightColor
            }
            else {
                root.color = Theme.backgroundColor
            }
        }

        // Child components get put here.
        Item {
            id: contentItem
            anchors.fill: parent
        }
    }

    Behavior on height {
        NumberAnimation {
            duration: {
                if (root.height > 0) {
                    return Theme.animationDuration
                }
                else {
                    // Usually, the initial size is often 0 and unnecessary animating happens when contents are rendered.
                    return 0
                }
            }
        }
    }

    Behavior on color {
        ColorAnimation {
            duration: Theme.animationDuration
        }
    }
}
