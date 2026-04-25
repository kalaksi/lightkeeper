/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import org.kde.syntaxhighlighting 1.0

import Lightkeeper 1.0

import ".."
import "../StyleOverride" as StyleOverride
import "../js/Utils.js" as Utils


Item {
    id: root
    required property string hostId
    required property string remoteFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0
    property string _detectedLanguage: Utils.detectLanguageFromPath(root.remoteFilePath)

    signal saved(commandId: string, remoteFilePath: string, content: string)
    signal closed(remoteFilePath: string)
    signal contentChanged(remoteFilePath: string, newContent: string)

    onRemoteFilePathChanged: {
        root._detectedLanguage = Utils.detectLanguageFromPath(root.remoteFilePath)
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === invocationId) {
                root.pendingInvocation = 0

                if (commandResult.criticality === "Normal") {
                    root.text = commandResult.message
                }

                root.activate()
            }
        }
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === ""
    }

    Flickable {
        id: editorFlickable
        visible: root.text !== ""
        anchors.fill: parent
        anchors.leftMargin: Theme.spacingLoose
        anchors.rightMargin: Theme.spacingNormal
        anchors.topMargin: Theme.spacingLoose
        anchors.bottomMargin: Theme.spacingLoose
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        contentWidth: width
        contentHeight: Math.max(textEdit.contentHeight, height)

        ScrollBar.vertical: StyleOverride.ScrollBar {
            policy: ScrollBar.AsNeeded
        }

        TextEdit {
            id: textEdit
            // Enabled only if all data is received.
            enabled: root.pendingInvocation === 0
            width: editorFlickable.width
            height: contentHeight
            wrapMode: Text.WordWrap
            color: Theme.textColor
            text: root.text
            font.family: "monospace"

            onTextChanged: root.activate()

            SyntaxHighlighter {
                id: syntaxHighlighter
                textEdit: textEdit
                definition: root._detectedLanguage || ""

                Component.onCompleted: {
                    let darkThemes = ["GitHub Dark", "Breeze Dark", "Solarized Dark"]
                    for (let i = 0; i < darkThemes.length; i++) {
                        let theme = Repository.theme(darkThemes[i])
                        if (theme.name !== "") {
                            syntaxHighlighter.theme = theme
                            return
                        }
                    }
                    syntaxHighlighter.theme = Repository.defaultTheme()
                }
            }
        }
    }

    Shortcut {
        sequence: StandardKey.Save
        onActivated: {
            root.save()
        }
    }

    function save() {
        if (root.commandId === "" || root.remoteFilePath === "") {
            return
        }

        let content = textEdit.text
        root.saved(root.commandId, root.remoteFilePath, content)
    }

    function activate() {
        // If still waiting for data, then content can't have changed yet.
        if (root.pendingInvocation === 0) {
            // Update save-button enabled-status.
            root.contentChanged(root.remoteFilePath, textEdit.text)
        }
    }

    function deactivate() {
        // Do nothing.
    }

    function refresh() {
        // DO nothing.
    }

    function close() {
        root.closed(root.remoteFilePath)
    }
}