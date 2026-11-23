/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0
import Lighthouse.InfoBalloon 1.0

import "../Button"
import "../StyleOverride"


RowLayout {
    id: root

    property string settingKey: ""
    // Current config value: placeholder, plaintext or empty.
    property string saveValue: ""
    // Core-connection SSH secrets use the local desktop store even with remote core.
    property bool useCoreSecretsBackend: false
    // Set when the user has typed or deleted. Parent stores `text` on save only if modified.
    property bool modified: false
    property alias text: field.text
    property bool _revealed: false

    readonly property string _infoText: root.useCoreSecretsBackend
        ? LK.config.coreSecretsFieldInfo(root.saveValue)
        : LK.config.secretsFieldInfo(root.saveValue)
    readonly property string _warningText: root.useCoreSecretsBackend
        ? LK.config.coreSecretsBackendWarning(root.saveValue)
        : LK.config.secretsBackendWarning(root.saveValue)

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

    ImageButton {
        id: infoButton
        imageSource: "qrc:/main/images/alert/info"
        flatButton: true
        roundButton: true
        size: field.implicitHeight * 0.8
        color: root._warningText !== "" ? Theme.criticalityColor("warning") : "transparent"
        onClicked: infoBalloon.toggle()

        Layout.preferredWidth: field.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    InfoBalloon {
        id: infoBalloon
        anchor: infoButton
        text: root._infoText
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        textColor: Theme.textColor
        contentPadding: Theme.spacingLoose
        edgeMargin: Theme.spacingNormal
        placementGap: Theme.spacingTight
        fontPointSize: Theme.fontSize
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
