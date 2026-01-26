/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import org.kde.syntaxhighlighting 1.0

import Theme

import ".."
import "../js/Utils.js" as Utils


Item {
    id: root
    required property string localFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0
    property string _detectedLanguage: Utils.detectLanguageFromPath(root.localFilePath)

    signal saved(commandId: string, localFilePath: string, content: string)
    signal closed(localFilePath: string)
    signal contentChanged(localFilePath: string, newContent: string)

    onLocalFilePathChanged: {
        root._detectedLanguage = Utils.detectLanguageFromPath(root.localFilePath)
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

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: rootScrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ScrollView {
        id: rootScrollView
        visible: root.text !== ""
        anchors.fill: parent
        anchors.margins: Theme.spacingLoose
        clip: true

        TextEdit {
            id: textEdit
            // Enabled only if all data is received.
            enabled: root.pendingInvocation === 0
            anchors.fill: parent
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
                    // Fallback to default.
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
        if (root.commandId === "" || root.localFilePath === "") {
            return
        }

        let content = textEdit.text
        root.saved(root.commandId, root.localFilePath, content)
    }

    function activate() {
        // If still waiting for data, then content can't have changed yet.
        if (root.pendingInvocation === 0) {
            // Update save-button enabled-status.
            root.contentChanged(root.localFilePath, textEdit.text)
        }
    }

    function deactivate() {
        // Do nothing.
    }

    function refresh() {
        // DO nothing.
    }

    function close() {
        root.closed(root.localFilePath)
    }
}