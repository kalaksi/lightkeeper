/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Theme


Item {
    id: root
    default property alias contentItem: contentItem.data
    property bool useRounding: true
    property bool firstItem: false
    property bool selected: false
    property int padding: 0
    implicitHeight: 40
    implicitWidth: parent.width

    signal clicked()

    // Stylish rounded cell for first item.
    Rectangle {
        id: rounded
        anchors.fill: parent
        radius: root.useRounding && root.firstItem ? 9 : 0
        color: root.getBackgroundColor(root.selected)

        MouseArea {
            anchors.fill: parent
            onClicked: root.clicked()
        }
    }

    Rectangle {
        color: root.getBackgroundColor(root.selected)
        width: rounded.radius
        anchors.top: rounded.top
        anchors.bottom: rounded.bottom
        anchors.right: rounded.right
    }

    Item {
        id: contentItem
        height: parent.height
        width: parent.width - root.padding
        anchors.centerIn: parent
    }

    function getBackgroundColor(selected) {
        if (selected === true) {
            return Qt.darker(palette.highlight)
        }
        else if (model.row % 2 == 0) {
            return Theme.alternateBaseColor
        }
        else {
            return Theme.baseColor
        }
    }
}