/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Lightkeeper 1.0

import ".."
import "../Text"

Item {
    id: root
    required property string hostId
    property string commandId: ""
    property var commandParams: []
    property var text: ""
    property var jsonText: ""
    property var errorText: ""
    property var criticality: ""
    property var pendingInvocation: 0
    property bool _initialFetchDone: false
    property bool _loading: pendingInvocation > 0


    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === invocationId) {
                root.pendingInvocation = 0

                // If message seems to contain JSON...
                if (commandResult.message.startsWith("{")) {
                    root.jsonText = commandResult.message
                    root.text = ""
                }
                else {
                    root.text = commandResult.message
                    root.jsonText = ""
                }

                root.errorText = commandResult.error
                root.criticality = commandResult.criticality
            }
        }
    }

    Rectangle {
        color: Theme.backgroundColor
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root._loading
    }

    ScrollView {
        visible: !root._loading && root.jsonText !== ""
        anchors.fill: parent

        JsonTextFormat {
            anchors.fill: parent
            anchors.margins: 20
            jsonText: root.jsonText
        }
    }

    ScrollView {
        visible: !root._loading && root.text !== ""
        anchors.fill: parent

        NormalText {
            anchors.fill: parent
            anchors.margins: 20
            wrapMode: Text.WordWrap
            textFormat: Text.MarkdownText
            text: root.text
        }
    }

    AlertText {
        text: root.errorText
        criticality: root.criticality
        visible: !root._loading && root.errorText !== ""
    }

    function refresh() {
        root.pendingInvocation = LK.command.executePlain(
            root.hostId,
            root.commandId,
            root.commandParams
        )
    }

    function activate() {
        if (!root._initialFetchDone) {
            root._initialFetchDone = true
            root.refresh()
        }
    }

    function deactivate() {
        // Do nothing.
    }

    function close() {
        // Do nothing.
    }
}
