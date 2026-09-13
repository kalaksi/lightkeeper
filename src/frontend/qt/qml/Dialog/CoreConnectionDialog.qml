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


LightkeeperDialog {
    id: root
    title: "Lightkeeper Core"
    implicitWidth: 560
    implicitHeight: 420
    standardButtons: Dialog.Close

    property string statusText: ""
    property string errorText: ""
    property bool busy: false
    property bool usingRemote: false

    onOpened: {
        root.loadProfile()
        root.refreshStatus()
    }

    Connections {
        target: LK

        function onCoreConnectionChanged() {
            root.refreshStatus()
        }
    }

    function loadProfile() {
        let profile = LK.config.getCoreConnection()
        hostField.text = profile.host || ""
        portField.text = profile.port || ""
        usernameField.text = profile.username || ""
    }

    function saveProfile() {
        // Always clear remoteSocketPath so the admin-host socket is auto-discovered over SSH.
        LK.config.setCoreConnection({
            host: hostField.text.trim(),
            port: portField.text.trim(),
            username: usernameField.text.trim(),
            remoteSocketPath: "",
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
            root.statusText = "Disconnected from remote core (restart app to use local mode)"
            root.errorText = error
        }
        else {
            root.statusText = "Using local backend"
            root.errorText = error
        }
    }

    function runTest() {
        root.busy = true
        root.errorText = ""
        root.saveProfile()
        let error = LK.probeCore()
        root.busy = false
        if (error === "") {
            root.statusText = "Probe succeeded"
            root.errorText = ""
        }
        else {
            root.statusText = "Probe failed"
            root.errorText = error
        }
    }

    function runConnect() {
        root.busy = true
        root.errorText = ""
        root.saveProfile()
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
                onClicked: root.runTest()
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

        NormalText {
            Layout.fillWidth: true
            text: root.statusText
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

        WorkingSprite {
            visible: root.busy
            Layout.alignment: Qt.AlignHCenter
        }
    }
}
