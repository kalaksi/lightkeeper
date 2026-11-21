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
    // Set when the user has typed or deleted. Parent stores `text` on save only if modified.
    property bool modified: false
    property alias text: field.text
    property bool _revealed: false

    spacing: Theme.spacingNormal

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

    // Only shows typed text. Stored secrets are never read back into the UI.
    ImageButton {
        imageSource: "qrc:/main/images/button/view-visible"
        size: field.implicitHeight * 0.8
        tooltip: root._revealed ? "Hide typed text" : "Show typed text"
        enabled: root.enabled && field.text !== ""
        onClicked: root._revealed = !root._revealed

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

    function placeholder() {
        if (root.modified) {
            return root.saveValue !== "" && field.text === "" ? "(will be removed)" : ""
        }
        if (root.saveValue !== "") {
            return "(stored)"
        }
        return root.enabled ? "" : "unset"
    }

    function reset() {
        field.text = ""
        root.modified = false
        root._revealed = false
    }
}
