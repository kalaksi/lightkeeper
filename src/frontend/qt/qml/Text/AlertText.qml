/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Theme


Item {
    id: root
    required property string text
    required property string criticality
    property real imageScale: 2.0

    implicitWidth: textContent.contentWidth + image.width
    implicitHeight: Math.max(textContent.implicitHeight, image.height)

    Row {
        id: row
        padding: 20
        spacing: 10
        anchors.verticalCenter: parent.verticalCenter

        Image {
            id: image
            antialiasing: true
            source: Theme.iconForCriticality(root.criticality)
            width: 22 * root.imageScale
            height: 22 * root.imageScale
            anchors.verticalCenter: parent.verticalCenter
        }

        NormalText {
            id: textContent
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            wrapMode: Text.Wrap
            width: root.width - image.width  - row.spacing - row.padding * 2
        }
    }
}
