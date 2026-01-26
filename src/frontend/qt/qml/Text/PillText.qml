/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import "../js/TextTransform.js" as TextTransform

Item {
    id: root
    property string text: ""
    property alias textColor: textElement.color
    property string pillColor: "#30FFFFFF"
    property int padding: 2

    implicitWidth: textElement.implicitWidth + padding * 2
    implicitHeight: textElement.implicitHeight

    Rectangle {
        color: root.pillColor
        anchors.fill: parent
        radius: parent.height
        visible: TextTransform.removeWhitespaces(textElement.text).length > 0
    }

    Text {
        id: textElement
        text: root.text
        anchors.fill: parent
        font.pointSize: 8
        verticalAlignment: Text.AlignVCenter
        horizontalAlignment: Text.AlignHCenter
        leftPadding: root.padding
        rightPadding: root.padding
    }
}