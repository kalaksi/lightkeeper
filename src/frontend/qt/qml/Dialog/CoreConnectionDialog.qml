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
    property string statusHint: ""
    property string errorText: ""
    property string probeSummary: ""
    property string probeRecommendation: ""
    property string connectionState: "disconnected"
    property bool busy: false
    property bool usingRemote: false
    property bool _loadingProfile: false
    property var _pendingPassword: null
    property var _pendingPassphrase: null
    property string _passwordSaveValue: ""
    property string _passphraseSaveValue: ""
    // "connect", "test", or "" — retry after trusting a host key.
    property string _pendingRetry: ""

    readonly property string statusCriticality: {
        if (root.busy
            || root.connectionState === "connecting_ssh"
            || root.connectionState === "handshaking"
            || root.connectionState === "reconnecting") {
            return "Warning"
        }
        if (root.connectionState === "connected") {
            return "Normal"
        }
        if (root.connectionState === "failed"
            || (root.usingRemote && root.connectionState !== "connected")) {
            return "Error"
        }
        return "Info"
    }

    // Opaque accent for the status banner (colorForCriticality is too transparent for fills).
    readonly property color statusAccentColor: {
        if (root.statusCriticality === "Normal") {
            return "#33cc33"
        }
        if (root.statusCriticality === "Warning") {
            return "#ffcc00"
        }
        if (root.statusCriticality === "Error") {
            return "#ff3300"
        }
        return Theme.borderColor
    }

    onOpened: {
        root._loadingProfile = true
        root.loadProfile()
        root._loadingProfile = false
        LK.clearCoreHostProbe()
        root.probeSummary = ""
        root.probeRecommendation = ""
        root.refreshStatus()
        if (root.offerHostKeyChallenge()) {
            root._pendingRetry = "connect"
        }
    }

    Connections {
        target: LK

        function onCoreConnectionChanged() {
            root.refreshStatus()
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
                root._pendingRetry = ""
                return
            }
            root.busy = true
            let error = LK.verifyCoreHostKey(challenge.keyId)
            root.busy = false
            if (error === "") {
                root.statusText = "Host key trusted"
                root.statusHint = ""
                root.errorText = ""
                LK.clearCoreHostKeyChallenge()
                let retry = root._pendingRetry
                root._pendingRetry = ""
                if (retry === "connect") {
                    root.runConnect()
                }
                else if (retry === "test") {
                    root.runCoreProbe()
                }
            }
            else {
                root.statusText = "Failed to trust host key"
                root.statusHint = "Fix the problem and try again"
                root.errorText = error
                root._pendingRetry = ""
            }
        }

        onRejected: {
            LK.clearCoreHostKeyChallenge()
            root._pendingRetry = ""
        }
    }

    contentItem: ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: statusColumn.implicitHeight + Theme.spacingNormal * 2
            color: Theme.baseColor
            border.color: Theme.borderColor
            border.width: 1
            radius: 4
            clip: true

            Rectangle {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: 4
                color: root.statusAccentColor
            }

            ColumnLayout {
                id: statusColumn
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: Theme.spacingNormal + 4
                anchors.rightMargin: Theme.spacingNormal
                spacing: Theme.spacingTight

                NormalText {
                    Layout.fillWidth: true
                    text: root.statusText
                    wrapMode: Text.WordWrap
                    font.bold: true
                    color: Theme.textColor
                }

                NormalText {
                    Layout.fillWidth: true
                    visible: root.statusHint.length > 0
                    text: root.statusHint
                    wrapMode: Text.WordWrap
                    color: Theme.textColor
                    opacity: 0.85
                }
            }
        }

        SmallText {
            Layout.fillWidth: true
            text: "Connect to lightkeeper-core on a remote host over SSH."
            wrapMode: Text.WordWrap
            color: Theme.textColorDark
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingNormal

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingLoose

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: Theme.spacingTight

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
                }

                ColumnLayout {
                    spacing: Theme.spacingTight

                    Label {
                        text: "Port"
                    }

                    TextField {
                        id: portField
                        Layout.preferredWidth: 72
                        placeholderText: "22"
                        placeholderTextColor: Theme.textColorDark
                        enabled: !root.busy
                        validator: IntValidator {
                            bottom: 1
                            top: 65535
                        }
                        onTextChanged: root.clearProbeCache()
                    }
                }
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
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingTight

            Label {
                text: "Authentication"
                font.bold: true
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 2
                columnSpacing: Theme.spacingNormal
                rowSpacing: Theme.spacingNormal

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
                    text: "Method"
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
                    delegate: ItemDelegate {
                        required property string label
                        width: methodCombo.width
                        text: label
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

                FilePathField {
                    id: privateKeyPathField
                    visible: methodCombo.currentValue === "key"
                    Layout.fillWidth: true
                    placeholderText: "Path to private key..."
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

            Item {
                Layout.fillWidth: true
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Theme.spacingTight
            visible: root.probeSummary.length > 0 || root.errorText.length > 0 || root.busy

            Label {
                visible: root.probeSummary.length > 0
                text: "Diagnostics"
                font.bold: true
            }

            NormalText {
                Layout.fillWidth: true
                visible: root.probeSummary.length > 0
                text: root.probeSummary
                wrapMode: Text.WordWrap
            }

            SmallText {
                Layout.fillWidth: true
                visible: root.probeRecommendation.length > 0
                text: root.probeRecommendation
                wrapMode: Text.WordWrap
                color: Theme.textColorDark
            }

            NormalText {
                Layout.fillWidth: true
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

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }

    function loadProfile() {
        let profile = LK.config.getCoreConnection()
        hostField.text = profile.host || ""
        portField.text = profile.port || ""
        usernameField.text = profile.username || ""
        autoConnectCheckBox.checked = !!profile.autoConnect
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
        root.probeRecommendation = ""
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
        })
    }

    function formattedHostLabel() {
        let host = hostField.text.trim()
        if (host === "") {
            return ""
        }
        let user = usernameField.text.trim()
        let port = portField.text.trim()
        let label = user !== "" ? (user + "@" + host) : host
        if (port !== "" && port !== "22") {
            label += ":" + port
        }
        return label
    }

    function refreshStatus() {
        let state = LK.getCoreConnectionState()
        let error = LK.getCoreConnectionError()
        root.usingRemote = LK.isUsingRemoteCore()
        root.connectionState = state
        let hostLabel = root.formattedHostLabel()

        if (state === "connected") {
            root.statusText = hostLabel !== ""
                ? "Connected to remote core - " + hostLabel
                : "Connected to remote core"
            root.statusHint = ""
            root.errorText = ""
        }
        else if (state === "connecting_ssh") {
            root.statusText = "Connecting over SSH..."
            root.statusHint = hostLabel
            root.errorText = error
        }
        else if (state === "handshaking") {
            root.statusText = "Authenticating with lightkeeper-core..."
            root.statusHint = hostLabel
            root.errorText = error
        }
        else if (state === "reconnecting") {
            root.statusText = "Reconnecting..."
            root.statusHint = hostLabel
            root.errorText = error
        }
        else if (state === "failed") {
            root.statusText = root.usingRemote ? "Remote connection failed" : "Connection failed"
            root.statusHint = "Fix settings and Connect again"
            root.errorText = error
        }
        else if (root.usingRemote) {
            root.statusText = "Remote core disconnected"
            root.statusHint = "Connect again, or Quit to use local"
            root.errorText = error
        }
        else {
            root.statusText = "Using local backend"
            root.statusHint = "Configure a remote host to connect"
            root.errorText = error
        }
    }

    function offerHostKeyChallenge() {
        let challenge = LK.getCoreHostKeyChallenge()
        if (!challenge || !challenge.keyId) {
            return false
        }
        let message = challenge.message || "Trust this host key?"
        hostKeyConfirmDialog.text = message + "\n\n" + challenge.keyId
        hostKeyConfirmDialog.open()
        return true
    }

    function refreshProbeSummary() {
        let probe = LK.getCoreHostProbe()
        if (!probe || !probe.architecture) {
            root.probeSummary = ""
            root.probeRecommendation = ""
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
        }
        else {
            lines.push("Binary: lightkeeper-core not found")
        }
        root.probeSummary = lines.join("\n")
    }

    function runCoreProbe() {
        root.busy = true
        root.errorText = ""
        root.probeRecommendation = ""
        root.saveProfile()
        let error = LK.probeCoreHost()
        root.busy = false
        root.refreshProbeSummary()
        if (error === "") {
            let probe = LK.getCoreHostProbe()
            if (probe.socketPath) {
                let coreError = LK.probeCore()
                if (coreError === "") {
                    root.statusText = "Probe succeeded"
                    root.statusHint = "SSH and core handshake are healthy"
                    root.errorText = ""
                    root.probeRecommendation = "Ready to Connect."
                }
                else if (root.offerHostKeyChallenge()) {
                    root._pendingRetry = "test"
                    root.statusText = "Host key verification required"
                    root.statusHint = "Trust the host key to continue"
                    root.errorText = coreError
                    root.probeRecommendation = ""
                }
                else {
                    root.statusText = "SSH ok; core handshake failed"
                    root.statusHint = "Check that lightkeeper-core is running"
                    root.errorText = coreError
                    root.probeRecommendation = "Socket is present but the core handshake failed."
                }
            }
            else if (probe.binaryPath) {
                root.statusText = "SSH ok; core socket missing"
                root.statusHint = "Is the lightkeeper-core service running?"
                root.errorText = ""
                root.probeRecommendation = "Binary found, but the service socket is not present."
            }
            else {
                root.statusText = "SSH ok; lightkeeper-core not installed"
                root.statusHint = "Install lightkeeper-core on the remote host"
                root.errorText = ""
                root.probeRecommendation = "Remote install from this dialog is not available yet."
            }
        }
        else if (root.offerHostKeyChallenge()) {
            root._pendingRetry = "test"
            root.statusText = "Host key verification required"
            root.statusHint = "Trust the host key to continue"
            root.errorText = error
            root.probeSummary = ""
            root.probeRecommendation = ""
        }
        else {
            root.statusText = "Probe failed"
            root.statusHint = "Fix settings and Test again"
            root.errorText = error
            root.probeSummary = ""
            root.probeRecommendation = ""
        }
    }

    function runConnect() {
        root.busy = true
        root.errorText = ""
        root.saveProfile()
        root.probeSummary = ""
        root.probeRecommendation = ""
        let error = LK.connectCore()
        root.busy = false
        root.refreshStatus()
        if (error !== "") {
            if (root.offerHostKeyChallenge()) {
                root._pendingRetry = "connect"
                root.statusText = "Host key verification required"
                root.statusHint = "Trust the host key to continue"
            }
            else {
                root.statusText = "Connect failed"
                root.statusHint = "Fix settings and Connect again"
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
