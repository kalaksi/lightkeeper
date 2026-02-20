/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Theme

import "../Text"
import "../js/Utils.js" as Utils


LightkeeperDialog {
    id: root

    property bool acceptOnly: false
    property bool keepHidden: false
    property string text: ""
    // Center short text automatically.
    property bool centerText: text.length < 40

    title: "Confirmation"
    standardButtons: acceptOnly ? Dialog.Ok : Dialog.Yes | Dialog.No
    implicitWidth: Utils.clamp(dialogText.implicitWidth, 300, 1000) + 100
    implicitHeight: Utils.clamp(dialogText.implicitHeight, 200, 600) + 50
    anchors.centerIn: parent

    contentItem: Item {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom

        NormalText {
            id: dialogText
            text: root.text
            width: parent.width
            wrapMode: Text.Wrap
            horizontalAlignment: root.centerText ? Text.AlignHCenter : Text.AlignLeft
        }
    }

    Component.onCompleted: {
        if (!keepHidden) {
            visible = true
        }
    }
}