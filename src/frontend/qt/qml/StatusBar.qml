/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "Misc"
import "Text"


ToolBar {
    id: root
    required property int errorCount
    required property int jobsLeft
    required property int hostCount
    padding: 0

    background: BorderRectangle {
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderTop: 1
    }

    RowLayout {
        height: jobsText.implicitHeight
        width: parent.width
        spacing: Theme.spacingLoose

        // Spacer
        Item {
            Layout.fillWidth: true
        }

        NormalText {
            id: hostCountText
            text: root.hostCount + " hosts"
            color: Theme.textColorDark
        }

        ToolSeparator {
            Layout.margins: parent.height / 6
            Layout.maximumHeight: parent.height - parent.height / 6
            Layout.alignment: Qt.AlignVCenter
        }

        NormalText {
            id: jobsText
            rightPadding: Theme.spacingLoose * 2
            text: root.jobsLeft + " jobs left"
            // This makes the text less prominent when there are no jobs left.
            // color: root.jobsLeft > 0 ? Theme.textColor : Theme.disabledTextColor
            color: Theme.textColorDark
        }
    }
}