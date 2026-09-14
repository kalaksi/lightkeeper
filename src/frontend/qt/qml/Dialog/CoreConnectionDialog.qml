/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Text"
import "../Misc"
import ".."


LightkeeperDialog {
    id: root
    title: "Lightkeeper Core"
    implicitWidth: 580
    implicitHeight: 640
    standardButtons: Dialog.Close

    property string statusText: ""
    property string errorText: ""
    property string probeSummary: ""
    property bool busy: false
    property bool usingRemote: false
    property bool _loadingProfile: false
    property bool installAvailable: false
    property var _pendingPassword: null
    property var _pendingPassphrase: null
    property string _passwordSaveValue: ""
    property string _passphraseSaveValue: ""

    onOpened: {
        root._loadingProfile = true
        root.loadProfile()
        root._loadingProfile = false
        LK.clearCoreHostProbe()
        LK.clearCoreHostKeyChallenge()
        root.probeSummary = ""
        root.installAvailable = false
        root.refreshStatus()
    }

    Connections {
        target: LK

        function onCoreConnectionChanged() {
            root.refreshStatus()
        }
    }

    ConfirmationDialog {
        id: installConfirmDialog
        keepHidden: true
        title: "Install lightkeeper-core"

        onAccepted: {
            root.statusText = "Remote install is not implemented yet"
            root.errorText = ""
        }
    }

    ConfirmationDialog {
        id: hostKeyConfirmDialog
        keepHidden: true
        title: "Verify host key"
        centerText: false

        onAccepted: {
            let challenge = LK.getCoreHostKeyChallenge()
            if (!challenge || !challenge.keyId) {
                root.errorText = "No pending host key challenge"
                return
            }
            root.busy = true
            let error = LK.verifyCoreHostKey(challenge.keyId)
            root.busy = false
            if (error === "") {
                root.statusText = "Host key trusted. Connect or Test again."
                root.errorText = ""
                LK.clearCoreHostKeyChallenge()
            }
            else {
                root.statusText = "Failed to trust host key"
                root.errorText = error
            }
        }

        onRejected: {
            LK.clearCoreHostKeyChallenge()
        }
    }

    contentItem: ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        NormalText {
            Layout.fillWidth: true
            text: "Connect to lightkeeper-core on a remote core host over SSH. Host keys use the desktop known_hosts file."
            wrapMode: Text.WordWrap
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Theme.spacingNormal
            rowSpacing: Theme.spacingNormal

            Label {
                text: "Host"
            }

            TextField {
                id: hostField
                Layout.fillWidth: true
                placeholderText: "remote-core.example.com"
                placeholderTextColor: Theme.textColorDark
                enabled: !root.busy
                onTextChanged: root.clearProbeCache()
            }

            Label {
                text: "Port"
            }

            TextField {
                id: portField
                Layout.fillWidth: true
                placeholderText: "22"
                placeholderTextColor: Theme.textColorDark
                enabled: !root.busy
                validator: IntValidator {
                    bottom: 1
                    top: 65535
                }
                onTextChanged: root.clearProbeCache()
            }

            Label {
                text: "Username"
            }

            TextField {
                id: usernameField
                Layout.fillWidth: true
                placeholderText: "current user"
                placeholderTextColor: Theme.textColorDark
                enabled: !root.busy
                onTextChanged: root.clearProbeCache()
            }

            Label {
                text: "Auth method"
            }

            ComboBox {
                id: methodCombo
                Layout.fillWidth: true
                enabled: !root.busy
                textRole: "label"
                valueRole: "methodId"
                model: ListModel {
                    ListElement { methodId: "agent"; label: "SSH agent" }
                    ListElement { methodId: "password"; label: "Password" }
                    ListElement { methodId: "key"; label: "Private key" }
                }
                onCurrentValueChanged: {
                    if (!root._loadingProfile) {
                        passwordField._revealedSecret = ""
                        passphraseField._revealedSecret = ""
                    }
                }
            }

            Label {
                visible: methodCombo.currentValue === "password"
                text: "Password"
            }

            SecretValueField {
                id: passwordField
                visible: methodCombo.currentValue === "password"
                enabled: !root.busy
                Layout.fillWidth: true
                settingKey: "password"
                saveValue: root._passwordSaveValue
                onRevealRequested: passwordField.revealSecret(root.resolveCoreSecret(passwordField))
                onEditRequested: passwordField.openEditor(root.resolveCoreSecret(passwordField))
                onSecretSubmitted: function(value, backend) {
                    root._pendingPassword = { value: value, backend: backend }
                    root.saveProfile()
                }
            }

            Label {
                visible: methodCombo.currentValue === "key"
                text: "Private key"
            }

            TextField {
                id: privateKeyPathField
                visible: methodCombo.currentValue === "key"
                Layout.fillWidth: true
                placeholderText: "path to private key"
                placeholderTextColor: Theme.textColorDark
                enabled: !root.busy
            }

            Label {
                visible: methodCombo.currentValue === "key"
                text: "Passphrase"
            }

            SecretValueField {
                id: passphraseField
                visible: methodCombo.currentValue === "key"
                enabled: !root.busy
                Layout.fillWidth: true
                settingKey: "private_key_passphrase"
                saveValue: root._passphraseSaveValue
                onRevealRequested: passphraseField.revealSecret(root.resolveCoreSecret(passphraseField))
                onEditRequested: passphraseField.openEditor(root.resolveCoreSecret(passphraseField))
                onSecretSubmitted: function(value, backend) {
                    root._pendingPassphrase = { value: value, backend: backend }
                    root.saveProfile()
                }
            }

            Label {
                visible: methodCombo.currentValue === "agent"
                text: "Agent key"
            }

            TextField {
                id: agentKeyField
                visible: methodCombo.currentValue === "agent"
                Layout.fillWidth: true
                placeholderText: "Optional key identifier"
                placeholderTextColor: Theme.textColorDark
                enabled: !root.busy
            }
        }

        CheckBox {
            id: verifyHostKeyCheckBox
            text: "Verify host key (known_hosts)"
            enabled: !root.busy
            Layout.fillWidth: true
        }

        CheckBox {
            id: autoConnectCheckBox
            text: "Connect automatically on startup"
            enabled: !root.busy
            Layout.fillWidth: true
            onCheckedChanged: {
                if (!root._loadingProfile) {
                    root.saveProfile()
                }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingNormal

            Button {
                text: "Test"
                enabled: !root.busy && hostField.text.trim().length > 0
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/network-disconnect"
                icon.height: 22
                icon.width: 22
                onClicked: root.runCoreProbe()
            }

            Button {
                text: "Connect"
                enabled: !root.busy && hostField.text.trim().length > 0
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/network-connect"
                icon.height: 22
                icon.width: 22
                onClicked: root.runConnect()
            }

            Button {
                text: "Disconnect"
                enabled: !root.busy && root.usingRemote
                onClicked: root.runDisconnect()
            }

            Button {
                text: "Install..."
                enabled: !root.busy && root.installAvailable
                onClicked: root.offerInstall()
            }

            Item {
                Layout.fillWidth: true
            }
        }

        NormalText {
            Layout.fillWidth: true
            text: root.statusText
            wrapMode: Text.WordWrap
        }

        NormalText {
            Layout.fillWidth: true
            visible: root.probeSummary.length > 0
            text: root.probeSummary
            wrapMode: Text.WordWrap
        }

        NormalText {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.errorText.length > 0
            text: root.errorText
            color: Theme.colorForCriticality("Error")
            wrapMode: Text.WrapAnywhere
        }

        Item {
            visible: root.busy
            Layout.fillWidth: true
            Layout.preferredHeight: 48

            WorkingSprite {
            }
        }
    }

    function loadProfile() {
        let profile = LK.config.getCoreConnection()
        hostField.text = profile.host || ""
        portField.text = profile.port || ""
        usernameField.text = profile.username || ""
        autoConnectCheckBox.checked = !!profile.autoConnect
        verifyHostKeyCheckBox.checked = profile.verifyHostKey !== false
        privateKeyPathField.text = profile.privateKeyPath || ""
        agentKeyField.text = profile.agentKeyIdentifier || ""
        root._passwordSaveValue = profile.password || ""
        root._passphraseSaveValue = profile.privateKeyPassphrase || ""
        root._pendingPassword = null
        root._pendingPassphrase = null
        passwordField._revealedSecret = ""
        passphraseField._revealedSecret = ""

        if ((profile.password || "") !== "") {
            methodCombo.currentIndex = methodCombo.indexOfValue("password")
        }
        else if ((profile.privateKeyPath || "") !== "") {
            methodCombo.currentIndex = methodCombo.indexOfValue("key")
        }
        else {
            methodCombo.currentIndex = methodCombo.indexOfValue("agent")
        }
    }

    function clearProbeCache() {
        if (root._loadingProfile) {
            return
        }
        LK.clearCoreHostProbe()
        root.probeSummary = ""
        root.installAvailable = false
    }

    function resolveCoreSecret(field) {
        let pending = field.settingKey === "password" ? root._pendingPassword : root._pendingPassphrase
        if (pending && pending.backend === "plaintext") {
            return pending.value || ""
        }
        if ((field.saveValue || "").indexOf("keyring:") === 0 || (field.saveValue || "").indexOf("pkeyring:") === 0) {
            return LK.config.getCoreSecret(field.settingKey) || ""
        }
        return field.saveValue || ""
    }

    function effectiveSecretValue(field, pending) {
        if (pending) {
            if (pending.backend === "plaintext") {
                return pending.value
            }
            return LK.config.storeCoreSecret(field.settingKey, pending.value)
        }
        return field.saveValue || ""
    }

    function saveProfile() {
        let method = methodCombo.currentValue
        let password = ""
        let privateKeyPath = ""
        let passphrase = ""
        let agentKey = ""

        if (method === "password") {
            password = root.effectiveSecretValue(passwordField, root._pendingPassword)
            root._passwordSaveValue = password
            root._pendingPassword = null
            if (root._passphraseSaveValue !== "") {
                LK.config.removeCoreSecret("private_key_passphrase")
                root._passphraseSaveValue = ""
            }
        }
        else if (method === "key") {
            privateKeyPath = privateKeyPathField.text.trim()
            passphrase = root.effectiveSecretValue(passphraseField, root._pendingPassphrase)
            root._passphraseSaveValue = passphrase
            root._pendingPassphrase = null
            if (root._passwordSaveValue !== "") {
                LK.config.removeCoreSecret("password")
                root._passwordSaveValue = ""
            }
        }
        else {
            agentKey = agentKeyField.text.trim()
            if (root._passwordSaveValue !== "") {
                LK.config.removeCoreSecret("password")
                root._passwordSaveValue = ""
            }
            if (root._passphraseSaveValue !== "") {
                LK.config.removeCoreSecret("private_key_passphrase")
                root._passphraseSaveValue = ""
            }
        }

        // Always clear remoteSocketPath so the remote core host socket is auto-discovered over SSH.
        LK.config.setCoreConnection({
            host: hostField.text.trim(),
            port: portField.text.trim(),
            username: usernameField.text.trim(),
            remoteSocketPath: "",
            autoConnect: autoConnectCheckBox.checked,
            password: password,
            privateKeyPath: privateKeyPath,
            privateKeyPassphrase: passphrase,
            agentKeyIdentifier: agentKey,
            verifyHostKey: verifyHostKeyCheckBox.checked,
            customKnownHostsPath: "",
        })
    }

    function refreshStatus() {
        let state = LK.getCoreConnectionState()
        let error = LK.getCoreConnectionError()
        root.usingRemote = LK.isUsingRemoteCore()

        if (state === "connected") {
            root.statusText = "Connected to remote core"
            root.errorText = ""
        }
        else if (state === "connecting_ssh") {
            root.statusText = "Connecting over SSH..."
            root.errorText = error
        }
        else if (state === "handshaking") {
            root.statusText = "Handshaking with core..."
            root.errorText = error
        }
        else if (state === "failed") {
            root.statusText = root.usingRemote ? "Remote connection failed" : "Connection failed"
            root.errorText = error
        }
        else if (root.usingRemote) {
            root.statusText = "Connect to remote Lightkeeper core or restart application to start using locally"
            root.errorText = error
        }
        else {
            root.statusText = "Using local backend"
            root.errorText = error
        }
    }

    function offerHostKeyChallenge() {
        let challenge = LK.getCoreHostKeyChallenge()
        if (!challenge || !challenge.keyId) {
            return false
        }
        hostKeyConfirmDialog.text = (challenge.message || "Trust this host key?") + "\n\n" + challenge.keyId
        hostKeyConfirmDialog.open()
        return true
    }

    function refreshProbeSummary() {
        let probe = LK.getCoreHostProbe()
        if (!probe || !probe.architecture) {
            root.probeSummary = ""
            root.installAvailable = false
            return
        }

        let lines = []
        lines.push(
            "Platform: " + (probe.osFlavor || probe.os || "Unknown")
            + " " + (probe.osVersion || "")
            + " / " + (probe.architecture || "Unknown")
        )
        if (probe.socketPath) {
            lines.push("Socket: " + probe.socketPath)
        }
        else {
            lines.push("Socket: not present")
        }
        if (probe.binaryPath) {
            lines.push("Binary: " + probe.binaryPath)
            root.installAvailable = false
        }
        else {
            lines.push("Binary: lightkeeper-core not found")
            root.installAvailable = true
        }
        root.probeSummary = lines.join("\n")
    }

    function buildInstallConfirmationText(probe) {
        let host = hostField.text.trim() || "remote core host"
        let lines = [
            "Install lightkeeper-core on " + host + " as a per-user service?",
            "",
            "These paths would be created or updated:",
            "  Binary: " + (probe.installBinaryPath || ""),
            "  Unit:   " + (probe.installUnitPath || ""),
            "  Socket: " + (probe.installSocketPath || "") + " (created when the service starts)",
            "",
            "Then: systemctl --user daemon-reload && enable --now lightkeeper-core",
            "",
            "Actual file transfer is not implemented yet; this confirms the planned layout.",
        ]
        return lines.join("\n")
    }

    function offerInstall() {
        let probe = LK.getCoreHostProbe()
        if (!probe || !probe.installBinaryPath) {
            root.errorText = "Run Test first to probe the remote core host"
            return
        }
        if (probe.binaryPath) {
            root.statusText = "lightkeeper-core is already present on the remote core host"
            return
        }

        installConfirmDialog.text = root.buildInstallConfirmationText(probe)
        installConfirmDialog.open()
    }

    function runCoreProbe() {
        root.busy = true
        root.errorText = ""
        root.installAvailable = false
        root.saveProfile()
        let error = LK.probeCoreHost()
        root.busy = false
        root.refreshProbeSummary()
        if (error === "") {
            let probe = LK.getCoreHostProbe()
            if (probe.socketPath) {
                let coreError = LK.probeCore()
                if (coreError === "") {
                    root.statusText = "Probe succeeded (SSH + core handshake)"
                    root.errorText = ""
                }
                else if (root.offerHostKeyChallenge()) {
                    root.statusText = "Host key verification required"
                    root.errorText = coreError
                }
                else {
                    root.statusText = "SSH ok; core socket present but handshake failed"
                    root.errorText = coreError
                }
            }
            else if (probe.binaryPath) {
                root.statusText = "SSH ok; binary found but core socket is missing (is the service running?)"
                root.errorText = ""
            }
            else {
                root.statusText = "SSH ok; lightkeeper-core is not installed on the remote core host"
                root.errorText = ""
                root.offerInstall()
            }
        }
        else if (root.offerHostKeyChallenge()) {
            root.statusText = "Host key verification required"
            root.errorText = error
            root.probeSummary = ""
            root.installAvailable = false
        }
        else {
            root.statusText = "Probe failed"
            root.errorText = error
            root.probeSummary = ""
            root.installAvailable = false
        }
    }

    function runConnect() {
        root.busy = true
        root.errorText = ""
        root.saveProfile()
        root.probeSummary = ""
        root.installAvailable = false
        let error = LK.connectCore()
        root.busy = false
        root.refreshStatus()
        if (error !== "") {
            if (root.offerHostKeyChallenge()) {
                root.statusText = "Host key verification required"
            }
            else {
                root.statusText = "Connect failed"
            }
            root.errorText = error
        }
    }

    function runDisconnect() {
        root.busy = true
        LK.disconnectCore()
        root.busy = false
        root.refreshStatus()
    }
}
