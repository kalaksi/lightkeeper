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
    implicitWidth: 560
    implicitHeight: 520
    standardButtons: Dialog.Close

    property string statusText: ""
    property string errorText: ""
    property string probeSummary: ""
    property bool busy: false
    property bool usingRemote: false
    property bool _loadingProfile: false
    property bool installAvailable: false

    onOpened: {
        root._loadingProfile = true
        root.loadProfile()
        root._loadingProfile = false
        LK.clearCoreHostProbe()
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

    contentItem: ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        NormalText {
            Layout.fillWidth: true
            text: "Connect to lightkeeper-core on an admin host over SSH (agent auth, known_hosts)."
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
                placeholderText: "admin-host.example.com"
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
    }

    function clearProbeCache() {
        if (root._loadingProfile) {
            return
        }
        LK.clearCoreHostProbe()
        root.probeSummary = ""
        root.installAvailable = false
    }

    function saveProfile() {
        // Always clear remoteSocketPath so the admin-host socket is auto-discovered over SSH.
        LK.config.setCoreConnection({
            host: hostField.text.trim(),
            port: portField.text.trim(),
            username: usernameField.text.trim(),
            remoteSocketPath: "",
            autoConnect: autoConnectCheckBox.checked,
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
        let host = hostField.text.trim() || "remote host"
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
            root.errorText = "Run Test first to probe the admin host"
            return
        }
        if (probe.binaryPath) {
            root.statusText = "lightkeeper-core is already present on the admin host"
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
                root.statusText = "SSH ok; lightkeeper-core is not installed on the admin host"
                root.errorText = ""
                root.offerInstall()
            }
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
            root.statusText = "Connect failed"
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
