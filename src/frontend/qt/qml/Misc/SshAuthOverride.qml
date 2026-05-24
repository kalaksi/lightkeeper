/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Text"
import "../StyleOverride"


// Host-level override editor for SSH connector username and authentication.
Item {
    id: root

    property string hostId: ""
    property string inheritedUsername: ""
    // ModuleSetting rows for auth keys (from LK.config.getHostConnectorModuleSettings).
    property var moduleSettings: []

    // ssh.settings keys this widget owns. The parent uses this to scrub
    // its own ssh.settings dict before merging in buildAuthFields().
    readonly property var ownedSshKeys: [
        "username", "password", "private_key_path", "private_key_passphrase", "agent_key_identifier"
    ]

    implicitHeight: layout.implicitHeight

    // Re-initialize whenever the parent reassigns moduleSettings (e.g. on dialog reopen).
    onModuleSettingsChanged: root._initFields()

    Component.onCompleted: root._initFields()

    ColumnLayout {
        id: layout
        anchors.left: parent.left
        anchors.right: parent.right
        spacing: 0

        Item {
            id: authFrame
            implicitHeight: groupColumn.implicitHeight + 2 * Theme.spacingNormal + sectionTitle.height / 2

            Layout.fillWidth: true
            Layout.topMargin: sectionTitle.height / 2

            Rectangle {
                anchors.fill: parent
                color: Theme.backgroundColor
                border.color: Theme.borderColor
                border.width: 1
            }

            Label {
                id: sectionTitle
                z: 1
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.top: parent.top
                anchors.topMargin: -height / 2
                padding: Theme.spacingTight
                text: "SSH authentication"
                background: Rectangle {
                    color: Theme.backgroundColor
                }
            }

            ColumnLayout {
                id: groupColumn
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: Theme.spacingNormal
                anchors.topMargin: Theme.spacingLoose
                spacing: Theme.spacingLoose

                SmallText {
                    Layout.fillWidth: true
                    text: "Host-level override for authentication settings. See also the defaults group."
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingNormal

                    ColumnLayout {
                        spacing: Theme.spacingTight

                        Layout.alignment: Qt.AlignTop

                        Label {
                            text: "Method"
                        }

                        ComboBox {
                            id: methodCombo
                            textRole: "label"
                            valueRole: "id"
                            model: [
                                { id: "inherit",  label: "From config groups" },
                                { id: "password", label: "Password" },
                                { id: "key",      label: "Private key" },
                                { id: "agent",    label: "SSH agent" },
                            ]
                            Layout.preferredWidth: 150
                            onCurrentValueChanged: {
                                passwordField._revealedSecret = ""
                                passphraseField._revealedSecret = ""
                            }
                        }
                    }

                    ColumnLayout {
                        spacing: Theme.spacingTight

                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignTop

                        ColumnLayout {
                            spacing: Theme.spacingTight

                            Layout.fillWidth: true

                            Label {
                                text: "Username"
                            }

                            TextField {
                                id: usernameField
                                Layout.fillWidth: true
                                placeholderText: root.inheritedUsername
                                placeholderTextColor: Theme.textColorDark
                            }
                        }

                        SecretValueField {
                            id: passwordField

                            // Pending edit: null = unchanged, otherwise { value, backend }.
                            property var _pending: null
                            property string _initialBackend: "keyring"
                            property string _initialValue: ""

                            visible: methodCombo.currentValue === "password"
                            settingKey: "password"
                            description: "Password for the SSH connection."
                            saveValue: root._effectiveSaveValue(passwordField)
                            backend: _pending !== null ? _pending.backend : _initialBackend
                            onRevealRequested: passwordField.revealSecret(root._resolveSecret(passwordField))
                            onEditRequested: passwordField.openEditor( root._resolveSecret(passwordField))
                            onSecretSubmitted: function(value, backend) {
                                if (value === undefined) {
                                    return
                                }
                                passwordField._pending = { value: value, backend: backend }
                            }

                            Layout.fillWidth: true
                        }

                        FilePathField {
                            id: keyPathField
                            visible: methodCombo.currentValue === "key"
                            placeholderText: "Path to private key..."
                            placeholderTextColor: Theme.textColorDark

                            Layout.fillWidth: true
                        }

                        SecretValueField {
                            id: passphraseField

                            // Pending edit: null = unchanged, otherwise { value, backend }.
                            property var _pending: null
                            property string _initialBackend: "keyring"
                            property string _initialValue: ""

                            visible: methodCombo.currentValue === "key"
                            settingKey: "private_key_passphrase"
                            description: "Passphrase for the private key file."
                            saveValue: root._effectiveSaveValue(passphraseField)
                            backend: _pending !== null ? _pending.backend : _initialBackend
                            onRevealRequested: passphraseField.revealSecret(root._resolveSecret(passphraseField))
                            onEditRequested: passphraseField.openEditor(root._resolveSecret(passphraseField))
                            onSecretSubmitted: function(value, backend) {
                                if (value === undefined) {
                                    return
                                }
                                passphraseField._pending = { value: value, backend: backend }
                            }

                            Layout.fillWidth: true
                        }

                        TextField {
                            id: agentIdField
                            visible: methodCombo.currentValue === "agent"
                            placeholderText: "Optional key identifier"
                            placeholderTextColor: Theme.textColorDark
                            selectByMouse: true

                            Layout.fillWidth: true
                        }
                    }
                }
            }
        }
    }

    // Returns only the auth-related fields this widget currently expresses,
    // as a partial dict to be merged into overrides.connectors.ssh.settings.
    // Secrets are NOT placed here; parent must call commitSecrets() to fill them in.
    function buildAuthFields() {
        let out = {}

        let username = usernameField.text.trim()
        if (username !== "") {
            out.username = username
        }

        let method = methodCombo.currentValue
        if (method === "key") {
            let path = keyPathField.text.trim()
            if (path !== "") {
                out.private_key_path = path
            }
        }
        else if (method === "agent") {
            let agentId = agentIdField.text.trim()
            if (agentId !== "") {
                out.agent_key_identifier = agentId
            }
        }
        return out
    }

    // Persists keyring writes/removals and mutates sshSettings in place to insert
    // password/passphrase values or placeholders for the chosen method. Removes secrets
    // for any method other than the active one.
    function commitSecrets(finalHostId, sshSettings) {
        let method = methodCombo.currentValue
        let activeKey = method === "password" ? "password"
            : method === "key" ? "private_key_passphrase" : ""

        for (let field of [passwordField, passphraseField]) {
            root._commitOneSecret(finalHostId, sshSettings, field, field.settingKey === activeKey)
        }
    }

    function _commitOneSecret(finalHostId, sshSettings, field, methodActive) {
        let hadKeyringSecret = field._initialBackend === "keyring" && field._initialValue !== ""

        if (!methodActive) {
            if (hadKeyringSecret) {
                LK.config.removeHostSecret(finalHostId, "ssh", field.settingKey)
            }
        }
        else if (field._pending === null) {
            // Untouched: keep original value (plaintext or keyring placeholder).
            if (field._initialValue !== "") {
                sshSettings[field.settingKey] = field._initialValue
            }
        }
        else if (field._pending.value === "") {
            if (hadKeyringSecret) {
                LK.config.removeHostSecret(finalHostId, "ssh", field.settingKey)
            }
        }
        else if (field._pending.backend === "keyring") {
            let placeholder = LK.config.storeHostSecret(finalHostId, "ssh", field.settingKey, field._pending.value)
            if (placeholder !== "") {
                sshSettings[field.settingKey] = placeholder
            }
        }
        else {
            sshSettings[field.settingKey] = field._pending.value
            if (hadKeyringSecret) {
                LK.config.removeHostSecret(finalHostId, "ssh", field.settingKey)
            }
        }
    }

    function _initFields() {
        let username = root._setting("username")
        usernameField.text = username.enabled ? username.value : ""

        let keyPath = root._setting("private_key_path")
        keyPathField.text = keyPath.enabled ? keyPath.value : ""

        let agentId = root._setting("agent_key_identifier")
        agentIdField.text = agentId.enabled ? agentId.value : ""

        methodCombo.currentIndex = root._detectInitialMethodIndex()

        let password = root._setting("password")
        passwordField._pending = null
        passwordField._initialValue = password.value
        passwordField._initialBackend = password.secretBackend

        let passphrase = root._setting("private_key_passphrase")
        passphraseField._pending = null
        passphraseField._initialValue = passphrase.value
        passphraseField._initialBackend = passphrase.secretBackend
    }

    function _setting(key) {
        for (let i = 0; i < root.moduleSettings.length; i++) {
            if (root.moduleSettings[i].key === key) {
                return root.moduleSettings[i]
            }
        }
        return { key: key, value: "", enabled: false, secretBackend: "keyring", isSecret: false }
    }

    function _detectInitialMethodIndex() {
        let keyPath = root._setting("private_key_path")
        if (keyPath.enabled && keyPath.value.trim() !== "") {
            return 2
        }
        let agentId = root._setting("agent_key_identifier")
        if (agentId.enabled && agentId.value.trim() !== "") {
            return 3
        }
        let password = root._setting("password")
        if (password.enabled && password.value !== "") {
            return 1
        }
        return 0
    }

    function _effectiveSaveValue(field) {
        if (field._pending !== null) {
            return field._pending.value !== "" ? "••••••" : ""
        }
        return field._initialValue !== "" ? "••••••" : ""
    }

    function _resolveSecret(field) {
        if (field._pending !== null) {
            return field._pending.value
        }
        if (field._initialValue === "") {
            return ""
        }
        if (field._initialBackend === "keyring") {
            return LK.config.getHostSecret(root.hostId, "ssh", field.settingKey) || ""
        }
        return field._initialValue
    }
}
