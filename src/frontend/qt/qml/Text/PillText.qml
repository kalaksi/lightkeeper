/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Lightkeeper 1.0

import "../js/TextTransform.js" as TextTransform

Item {
    id: root
    property string text: ""
    property alias textColor: textElement.color
    property string pillColor: "#30FFFFFF"
    property int padding: 4
    property string tooltip: ""

    implicitWidth: textElement.implicitWidth + padding * 2
    implicitHeight: textElement.implicitHeight + 2

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }

    ToolTip.visible: root.tooltip !== "" && mouseArea.containsMouse
    ToolTip.delay: Theme.tooltipDelay
    ToolTip.text: root.tooltip

    Rectangle {
        color: root.pillColor
        anchors.fill: parent
        radius: parent.height
        visible: TextTransform.removeWhitespaces(textElement.text).length > 0
    }

    Text {
        id: textElement
        text: root.text
        color: Theme.textColor
        anchors.centerIn: parent
        font.pointSize: 8
    }
}