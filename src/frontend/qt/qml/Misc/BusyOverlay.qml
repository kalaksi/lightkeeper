/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Lightkeeper 1.0

import ".."


Item {
    id: root
    property string text: ""

    z: 100

    Rectangle {
        anchors.fill: parent
        color: Theme.backgroundColor
        opacity: 0.55
    }

    WorkingSprite {
        show: root.visible
        text: root.text
    }

    MouseArea {
        anchors.fill: parent
    }
}
