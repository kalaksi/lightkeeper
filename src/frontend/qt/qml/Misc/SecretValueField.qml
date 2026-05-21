/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Button"
import "../Dialog"
import "../StyleOverride"


RowLayout {
    id: root

    property string settingKey: ""
    property string description: ""
    property string saveValue: ""
    property string backend: "keyring"
    property string _revealedSecret: ""

    spacing: Theme.spacingNormal

    signal revealRequested()
    signal editRequested()
    signal secretSubmitted(string value, string backend)

    TextField {
        id: maskedField
        visible: root._revealedSecret === ""
        enabled: root.enabled
        readOnly: true
        selectByMouse: false
        text: root.saveValue
        echoMode: TextInput.Password
        placeholderText: root.enabled ? "" : "unset"
        placeholderTextColor: Theme.textColorDark

        Layout.fillWidth: true
        Layout.alignment: Qt.AlignVCenter
    }

    TextField {
        visible: root._revealedSecret !== ""
        enabled: root.enabled
        readOnly: true
        selectByMouse: false
        text: root._revealedSecret

        Layout.fillWidth: true
        Layout.alignment: Qt.AlignVCenter
    }

    ImageButton {
        imageSource: "qrc:/main/images/button/view-visible"
        size: maskedField.implicitHeight * 0.8
        tooltip: root._revealedSecret !== "" ? "Hide password" : "Show password"
        enabled: root.enabled
        onClicked: {
            if (root._revealedSecret !== "") {
                root._revealedSecret = ""
            }
            else {
                root.revealRequested()
            }
        }

        Layout.preferredWidth: maskedField.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    ImageButton {
        imageSource: "qrc:/main/images/button/entry-edit"
        size: maskedField.implicitHeight * 0.8
        tooltip: "Edit secret"
        enabled: root.enabled
        onClicked: root.editRequested()

        Layout.preferredWidth: maskedField.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    SecretEditDialog {
        id: secretDialog

        onSecretSubmitted: function(value, backend) {
            root.secretSubmitted(value, backend)
            root._revealedSecret = ""
        }
    }

    function revealSecret(value) {
        root._revealedSecret = value
    }

    function openEditor(initialValue) {
        secretDialog.settingKey = root.settingKey
        secretDialog.description = root.description
        secretDialog.initialBackend = root.backend
        secretDialog.initialValue = initialValue
        secretDialog.open()
    }
}
