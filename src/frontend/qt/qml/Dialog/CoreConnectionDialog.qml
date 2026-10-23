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
    implicitHeight: 800
    standardButtons: Dialog.Close

    property string statusText: ""
    property string statusHint: ""
    property string errorText: ""
    property string probeSummary: ""
    property string probeRecommendation: ""
    property string coreStatus: ""
    property string connectionState: "disconnected"
    property bool busy: false
    property bool usingRemote: false
    property bool canInstall: false
    // Install replaces an existing user-local core with the version matching this desktop.
    property bool installIsUpgrade: false
    property bool _loadingProfile: false
    property var _pendingPassword: null
    property var _pendingPassphrase: null
    property string _passwordSaveValue: ""
    property string _passphraseSaveValue: ""
    // "connect", "test", or "" - retry after trusting a host key.
    property string _pendingRetry: ""

    readonly property string statusCriticality: {
        if (root.busy
            || root.connectionState === "connecting_ssh"
            || root.connectionState === "handshaking") {
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
        root.canInstall = false
        root.installIsUpgrade = false
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

    ConfirmationDialog {
        id: installConfirmDialog
        keepHidden: true
        title: root.installIsUpgrade ? "Upgrade lightkeeper-core" : "Install lightkeeper-core"
        centerText: false

        onAccepted: root.runInstall()
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

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingNormal

            Image {
                source: "qrc:/main/images/core-topology"
                sourceSize.width: 160
                sourceSize.height: 96
            }

            SmallText {
                Layout.fillWidth: true
                text: "Connect to lightkeeper-core on a remote host.<br>"
                    + "Lightkeeper-core is the backend that runs the monitoring and commands,"
                    + " while this application is the user interface."
                    + " Use a host that can reach your monitored hosts, e.g. a jump host or an always-on server. "
                    + "<b>Lightkeeper-core will listen on a unix socket and not on any port.</b>"
                textFormat: Text.RichText
                wrapMode: Text.WordWrap
                color: Theme.textColorDark
            }
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
                        placeholderText: "remote-core.example.invalid"
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

            ColumnLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingTight

                Label {
                    text: "Socket path"
                }

                TextField {
                    id: socketPathField
                    Layout.fillWidth: true
                    placeholderText: "~/.local/share/lightkeeper/core.sock"
                    placeholderTextColor: Theme.textColorDark
                    enabled: !root.busy
                    onTextChanged: root.clearProbeCache()
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
                Layout.preferredHeight: 34
                enabled: !root.busy && hostField.text.trim().length > 0
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/search"
                icon.height: 22
                icon.width: 22
                onClicked: root.runCoreProbe()
            }

            Button {
                text: "Connect"
                Layout.preferredHeight: 34
                enabled: !root.busy && hostField.text.trim().length > 0
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/network-connect"
                icon.height: 22
                icon.width: 22
                onClicked: root.runConnect()
            }

            Button {
                text: root.installIsUpgrade ? "Upgrade" : "Install"
                Layout.preferredHeight: 34
                enabled: !root.busy && root.canInstall
                onClicked: {
                    let probe = LK.getCoreHostProbe()
                    installConfirmDialog.text = "Install Lightkeeper-core " + LK.config.getVersion()
                        + " on this remote core host as a systemd user service in user's home directory?"
                        + (root.installIsUpgrade ? " The running service will be restarted." : "") + "\n\n"
                        + "Binary: " + probe.installBinaryPath + "\n"
                        + "Unit: " + probe.installUnitPath + "\n"
                        + "Socket: " + probe.installSocketPath
                    installConfirmDialog.open()
                }
            }

            Button {
                text: "Disconnect"
                Layout.preferredHeight: 34
                enabled: !root.busy && root.usingRemote
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/network-disconnect"
                icon.height: 22
                icon.width: 22
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

            Label {
                visible: root.probeSummary.length > 0
                text: root.coreStatus ? "Core status: " + root.coreStatus : "Core status"
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
                textFormat: Text.RichText
                wrapMode: Text.WordWrap
                color: Theme.textColorDark
                onLinkActivated: (link) => Qt.openUrlExternally(link)

                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.NoButton
                    cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.ArrowCursor
                }
            }

            NormalText {
                Layout.fillWidth: true
                visible: root.errorText.length > 0
                text: root.errorText
                color: Theme.colorForCriticality("Error")
                wrapMode: Text.WrapAnywhere
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
        socketPathField.text = profile.remoteSocketPath || ""
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
        root.canInstall = false
        root.installIsUpgrade = false
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

        // Empty remoteSocketPath means the remote core host socket is auto-discovered over SSH.
        LK.config.setCoreConnection({
            host: hostField.text.trim(),
            port: portField.text.trim(),
            username: usernameField.text.trim(),
            remoteSocketPath: socketPathField.text.trim(),
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
        else if (state === "failed") {
            root.statusText = root.usingRemote ? "Remote connection failed" : "Connection failed"
            root.statusHint = "Fix settings and Connect again"
            root.errorText = error
        }
        else if (root.usingRemote) {
            root.statusText = "Remote core disconnected"
            root.statusHint = "Connect again, or Disconnect to restart locally"
            root.errorText = error
        }
        else {
            root.statusText = "Currently using local backend"
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
        lines.push("Can run user service on background: "
            + (probe.lingerEnabled ? "yes" : "no (use \"loginctl enable-linger\" to allow)"))
        root.probeSummary = lines.join("\n")
    }

    function runCoreProbe() {
        root.busy = true
        root.errorText = ""
        root.probeRecommendation = ""
        root.coreStatus = ""
        root.canInstall = false
        root.installIsUpgrade = false
        root.saveProfile()
        let error = LK.probeCoreHost()
        root.busy = false
        root.refreshProbeSummary()
        if (error === "") {
            let probe = LK.getCoreHostProbe()
            // Install and upgrade only set up the default socket path.
            let customSocketPath = socketPathField.text.trim() !== ""
            if (probe.socketPath) {
                let coreError = LK.probeCore()
                if (coreError === "") {
                    root.statusText = "SSH connection works"
                    root.statusHint = ""
                    root.errorText = ""
                    root.probeRecommendation = "Ready to Connect."
                    root.coreStatus = "installed"
                }
                else if (root.offerHostKeyChallenge()) {
                    root._pendingRetry = "test"
                    root.statusText = "Host key verification required"
                    root.statusHint = "Trust the host key to continue"
                    root.errorText = coreError
                    root.probeRecommendation = ""
                }
                else if (LK.isCoreVersionMismatch()) {
                    root.statusText = "SSH connection works"
                    root.statusHint = ""
                    root.errorText = coreError
                    root.coreStatus = "different version installed"
                    if (customSocketPath) {
                        root.probeRecommendation = "Upgrade the lightkeeper-core serving the configured socket "
                            + "path, e.g. by updating the container image."
                    }
                    else if (probe.binaryPath === probe.installBinaryPath) {
                        root.probeRecommendation = "Use Upgrade to install the matching user-local version."
                        root.canInstall = true
                        root.installIsUpgrade = true
                    }
                    else {
                        root.probeRecommendation = "Upgrade lightkeeper-core at " + probe.binaryPath
                            + " with the method it was installed with, e.g. the system package manager."
                    }
                }
                else {
                    root.statusText = "SSH connection works"
                    root.statusHint = ""
                    root.errorText = coreError
                    root.probeRecommendation = "Socket is present but the core handshake failed."
                    root.coreStatus = "installed, not responding"
                }
            }
            else if (customSocketPath) {
                root.statusText = "SSH connection works"
                root.statusHint = ""
                root.errorText = ""
                root.probeRecommendation = "Socket is not present at the configured path. Install is only "
                    + "available with the default socket path, so clear the field to use it."
                root.coreStatus = "not running"
            }
            else if (probe.binaryPath) {
                root.statusText = "SSH connection works"
                root.statusHint = ""
                root.errorText = ""
                root.probeRecommendation = "Binary found, but the service socket is not present."
                root.coreStatus = "installed, not running"
            }
            else {
                root.statusText = "SSH connection works"
                root.statusHint = ""
                root.errorText = ""
                root.probeRecommendation = "Use 'install' to set up lightkeeper-core as a systemd user service "
                    + "in user's home directory.<br>"
                    + "Alternatively, run the ghcr.io/kalaksi/lightkeeper-core container. "
                    + "See <a href=\"https://github.com/kalaksi/lightkeeper/blob/master/packaging/lightkeeper-core/docker-compose.yml\">"
                    + "https://github.com/kalaksi/lightkeeper/blob/master/packaging/lightkeeper-core/docker-compose.yml</a>."
                root.coreStatus = "not installed"
                root.canInstall = true
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

    function runInstall() {
        root.busy = true
        root.errorText = ""
        let error = LK.installCore()
        root.busy = false
        if (error === "") {
            root.runCoreProbe()
        }
        else {
            root.statusText = "Install failed"
            root.statusHint = "Fix the problem and try again"
            root.errorText = error
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
            else if (LK.isCoreVersionMismatch()) {
                root.statusText = "lightkeeper-core version mismatch"
                root.statusHint = "Use Test to see upgrade options"
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
