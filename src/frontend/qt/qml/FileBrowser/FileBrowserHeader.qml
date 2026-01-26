/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    height: rowHeight
    color: headerColor

    required property var columnWidthProvider

    property int rowHeight: 28
    property int arrowWidth: 20
    property var columnHeaders: ["Column 1", "Column 2"]
    property color headerColor: palette.alternateBase

    Row {
        anchors.fill: parent
        anchors.leftMargin: root.arrowWidth

        Label {
            width: root.columnWidthProvider(0, root.width) - root.arrowWidth
            text: "Name"
            font: palette.buttonText
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }

        Repeater {
            model: root.columnHeaders.length

            Label {
                required property int index

                width: root.columnWidthProvider(index + 3, root.width)
                text: root.columnHeaders[index]
                font: palette.buttonText
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }
        }
    }
}

