/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Button"
import "../StyleOverride"


RowLayout {
    id: root

    property string settingKey: ""
    // Current config value: placeholder, plaintext or empty.
    property string saveValue: ""
    property string warningText: ""
    property bool allowReveal: true
    // Set when the user has typed or deleted. Parent stores `text` on save only if modified.
    property bool modified: false
    property alias text: field.text
    property bool _revealed: false

    spacing: Theme.spacingNormal

    signal revealRequested()

    TextField {
        id: field
        enabled: root.enabled
        selectByMouse: true
        echoMode: root._revealed ? TextInput.Normal : TextInput.Password
        placeholderText: root.placeholder()
        placeholderTextColor: Theme.textColorDark
        onTextEdited: root.modified = true

        Layout.fillWidth: true
        Layout.alignment: Qt.AlignVCenter
    }

    ImageButton {
        visible: root.allowReveal
        imageSource: "qrc:/main/images/button/view-visible"
        size: field.implicitHeight * 0.8
        tooltip: root._revealed ? "Hide secret" : "Show secret"
        enabled: root.enabled
        onClicked: {
            root._revealed = !root._revealed
            if (!root.modified) {
                field.text = ""
                if (root._revealed && root.saveValue !== "") {
                    root.revealRequested()
                }
            }
        }

        Layout.preferredWidth: field.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    ImageButton {
        imageSource: "qrc:/main/images/button/delete"
        size: field.implicitHeight * 0.8
        tooltip: "Remove secret"
        enabled: root.enabled && (field.text !== "" || (!root.modified && root.saveValue !== ""))
        onClicked: {
            field.text = ""
            root.modified = true
        }

        Layout.preferredWidth: field.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    ImageButton {
        id: infoButton
        imageSource: "qrc:/main/images/alert/info"
        size: field.implicitHeight * 0.8
        color: root.warningText !== "" ? Theme.criticalityColor("warning") : "transparent"
        onClicked: infoTip.visible ? infoTip.close() : infoTip.open()

        Layout.preferredWidth: field.implicitHeight
        Layout.alignment: Qt.AlignVCenter

        ToolTip {
            id: infoTip
            parent: infoButton
            delay: 0
            timeout: -1
            text: "Secrets use the backend configured in Preferences."
                + (root.warningText !== "" ? "\n" + root.warningText : "")
        }
    }

    function placeholder() {
        if (root.modified) {
            return root.saveValue !== "" && field.text === "" ? "(will be removed)" : ""
        }
        if (root.saveValue !== "") {
            return "(stored)"
        }
        return root.enabled ? "" : "unset"
    }

    function revealSecret(value) {
        field.text = value
    }

    function reset() {
        field.text = ""
        root.modified = false
        root._revealed = false
    }
}
