/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import ".."
import "../Button"
import "../Misc"
import "../js/Utils.js" as Utils


Item {
    id: root
    required property string hostId
    required property string remoteFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0
    property var textEditorItem: null
    property bool disableSaveButton: true
    property string editMode: "regular"
    property int fontSize: 12
    property bool wordWrap: true
    property var _aceEditorObject: null
    property bool _useSimpleCodeEditor: false
    property bool _vimCloseAfterSave: false
    property bool _saveOverlayActive: false
    property string _detectedLanguage: Utils.detectLanguageFromPath(root.remoteFilePath)
    property string _aceMode: Utils.mapLanguageToAceMode(root._detectedLanguage)
    // First time text was changed in editor.
    property bool _initialOpen: true

    signal saved(commandId: string, remoteFilePath: string, content: string)
    signal closed(remoteFilePath: string)
    signal contentChanged(remoteFilePath: string, newContent: string)
    // When closing the tab from Vim ex-commands (:q / :wq) or Emacs C-x C-c.
    signal closeTabRequested()

    onRemoteFilePathChanged: {
        root._detectedLanguage = Utils.detectLanguageFromPath(root.remoteFilePath)
        root._aceMode = Utils.mapLanguageToAceMode(root._detectedLanguage)
    }

    onTextChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.content = root.text
        }
        if (root._initialOpen) {
            root._initialOpen = false
            root._aceEditorObject.resetCursor()
        }
    }

    on_AceModeChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.mode = root._aceMode
        }
    }

    onEditModeChanged: {
        if (!root._useSimpleCodeEditor) {
            root._setEditorKeybindings()
        }
    }

    onFontSizeChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.setEditorOption("fontSize", root.fontSize)
        }
    }

    onWordWrapChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.wordWrap = root.wordWrap
        }
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === invocationId) {
                root.pendingInvocation = 0
                root._saveOverlayActive = false

                let closeAfterSave = root._vimCloseAfterSave
                root._vimCloseAfterSave = false

                if (commandResult.criticality === "Normal") {
                    root.text = commandResult.message
                }

                if (commandResult.criticality === "Normal" || commandResult.criticality === "Info") {
                    if (closeAfterSave) {
                        root.closeTabRequested()
                    }
                }
                else {
                    if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
                        root._aceEditorObject.showVimNotification(
                            "Operation failed",
                            Theme.criticalityColor(commandResult.criticality))
                    }
                }

                root.activate()
            }
        }
    }

    Component.onCompleted: {
        root._updateUseSimpleCodeEditor()
        root._loadEditorPreferences()
        
        if (!root._useSimpleCodeEditor) {
            aceEditorContainer.createEditor()
        }
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ToolBar {
        id: topBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: 0
        anchors.bottomMargin: Theme.spacingLoose
        height: 36
        visible: root.text !== ""

        background: BorderRectangle {
            backgroundColor: Theme.backgroundColor
            borderColor: Theme.borderColor
            borderBottom: 1
        }

        RowLayout {
            width: parent.width
            height: parent.height
            anchors.top: parent.top
            spacing: Theme.spacingNormal

            ToolButton {
                icon.source: "qrc:/main/images/button/document-save"
                text: "Save"
                display: AbstractButton.IconOnly
                onClicked: root.save()
                enabled: !root.disableSaveButton
                icon.height: 24
                icon.width: 24
                padding: 4
            }

            ToolSeparator {
            }

            Text {
                text: "Font"
                color: Theme.textColor
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            ComboBox {
                id: fontSizeComboBox
                model: [10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24]
                currentIndex: {
                    let index = model.indexOf(root.fontSize)
                    return index >= 0 ? index : 2
                }
                onActivated: function(index) {
                    root.fontSize = model[index]
                    root._saveEditorPreferences()
                }
                Layout.preferredWidth: 80
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            ToolSeparator {
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            Text {
                text: "Word wrap"
                color: Theme.textColor
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            Switch {
                checked: root.wordWrap
                onClicked: {
                    root.wordWrap = checked
                    root._saveEditorPreferences()
                }
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            Item {
                Layout.fillWidth: true
            }

            Text {
                text: "Edit mode"
                color: Theme.textColor
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            ComboBox {
                id: editModeComboBox
                model: ["regular", "vim", "emacs"]
                currentIndex: {
                    switch (root.editMode) {
                        case "regular": return 0
                        case "vim": return 1
                        case "emacs": return 2
                        default: return 0
                    }
                }
                onActivated: function(index) {
                    root.editMode = model[index]
                    root._saveEditorPreferences()
                }
                Layout.preferredWidth: 120
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }
        }
    }

    Item {
        id: aceEditorContainer
        anchors.top: topBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.text !== "" && root._aceEditorObject !== null && !root._useSimpleCodeEditor

        function createEditor() {
            if (root._aceEditorObject !== null || root._useSimpleCodeEditor) {
                return
            }
            
            // Create the AceEditor component dynamically.
            // This will return null if Lighthouse.AceEditor import is not available.
            let aceEditorQml = `
                import QtQuick;
                import QtWebEngine;
                import Lightkeeper 1.0;
                import Lighthouse.AceEditor 1.0;
                AceEditor {
                    anchors.fill: parent;
                    property var rootItem: null;
                    // palette.base
                    defaultBackgroundColor: Theme.baseColor
                    customScrollbarStyling: true
                    // palette.base
                    scrollbarTrackColor: Theme.baseColor
                    // palette.light
                    scrollbarThumbColor: "#474d54"
                    // palette.midlight
                    scrollbarThumbHoverColor: "#3a4045"
                    scrollbarSize: 8

                    onEditorContentChanged: function(newContent) {
                        if (rootItem) {
                            rootItem.contentChanged(rootItem.remoteFilePath, newContent);
                            rootItem.disableSaveButton = !LK.command.hasFileChanged(rootItem.hostId, rootItem.remoteFilePath, newContent);
                        }
                    }

                    onWriteRequested: function() {
                        if (rootItem) {
                            rootItem.save()
                        }
                    }

                    onQuitRequested: function(writeChanges, discardUnsaved, writeOnlyIfModified) {
                        if (!rootItem) {
                            return
                        }

                        if (writeChanges) {
                            if (!writeOnlyIfModified || (writeOnlyIfModified && !rootItem.disableSaveButton)) {
                                rootItem._vimCloseAfterSave = true
                                if (!rootItem.save()) {
                                    rootItem._vimCloseAfterSave = false
                                }
                            }
                        }
                        else if (!rootItem.disableSaveButton) {
                            if (discardUnsaved) {
                                rootItem._vimCloseAfterSave = false
                                rootItem.closeTabRequested()
                            }
                            else {
                                rootItem._aceEditorObject.showVimNotification(
                                    "There are unsaved changes",
                                    Theme.criticalityColor("Warning"))
                            }
                        }
                        else {
                            rootItem.closeTabRequested()
                        }
                    }
                }`

            let editorObject = Qt.createQmlObject(aceEditorQml, aceEditorContainer, "aceEditor")
            if (editorObject !== null) {
                root._aceEditorObject = editorObject
                editorObject.rootItem = root
                editorObject.wordWrap = root.wordWrap

                editorObject.editorReady.connect(function() {
                    root._setEditorKeybindings()
                    root._aceEditorObject.setEditorOption("fontSize", root.fontSize)
                    root._aceEditorObject.content = root.text
                    root._aceEditorObject.mode = root._aceMode
                    root._aceEditorObject.theme = "tomorrow_night"

                    root._aceEditorObject.resetCursor()
                })
            } else {
                console.log("Ace editor not available: failed to create")
            }
        }
    }

    Rectangle {
        id: simpleEditorBorder
        anchors.top: topBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.text !== "" && (root._aceEditorObject === null || root._useSimpleCodeEditor)
        color: Theme.backgroundColorLight
        border.width: 1
        border.color: Theme.borderColor

        Loader {
            id: textEditorLoader
            anchors.fill: parent
            anchors.margins: 1
            sourceComponent: (root._aceEditorObject === null || root._useSimpleCodeEditor) ? textEditorComponent : null

            onItemChanged: {
                if (item === null) {
                    root.textEditorItem = null
                }
            }

            Component {
                id: textEditorComponent

                HostDetailsTextEditorView {
                    id: textEditorInstance
                    hostId: root.hostId
                    remoteFilePath: root.remoteFilePath
                    text: root.text
                    commandId: root.commandId
                    pendingInvocation: root.pendingInvocation

                    Component.onCompleted: {
                        root.textEditorItem = textEditorInstance
                    }

                    onSaved: function(commandId, remoteFilePath, content) {
                        root.saved(commandId, remoteFilePath, content)
                    }
                    onClosed: function(remoteFilePath) {
                        root.closed(remoteFilePath)
                    }
                    onContentChanged: function(remoteFilePath, newContent) {
                        root.contentChanged(remoteFilePath, newContent)
                        root.disableSaveButton = !LK.command.hasFileChanged(root.hostId, remoteFilePath, newContent)
                    }
                }
            }
        }
    }

    Text {
        id: errorMessage
        visible: root._aceEditorObject === null && !root._useSimpleCodeEditor
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingLoose
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        color: Theme.textColor || "red"
        text: "Integrated code editor could not be loaded. Qt web engine or code editor QML component is likely missing.\n"+
              "You are using a simple text editor instead. To continue using simple editor without this warning, choose \"internal (simple)\" in settings."
    }

    BusyOverlay {
        anchors.fill: parent
        visible: root._saveOverlayActive
        text: "Saving..."
    }

    Shortcut {
        sequence: StandardKey.Save
        onActivated: {
            root.save()
        }
    }

    function _updateUseSimpleCodeEditor() {
        let preferences = LK.config.getPreferences()
        let textEditor = preferences.textEditor
        root._useSimpleCodeEditor = textEditor === "internal-simple"
    }

    function _loadEditorPreferences() {
        let preferences = LK.config.getPreferences()
        if (preferences.editorPreferences) {
            if (preferences.editorPreferences.editMode) {
                root.editMode = preferences.editorPreferences.editMode
            }
            if (preferences.editorPreferences.fontSize) {
                root.fontSize = preferences.editorPreferences.fontSize
            }
            if (preferences.editorPreferences.wordWrap !== undefined) {
                root.wordWrap = preferences.editorPreferences.wordWrap
            }
        }
    }

    function _saveEditorPreferences() {
        let preferences = LK.config.getPreferences()
        preferences.editorPreferences = preferences.editorPreferences
        preferences.editorPreferences.editMode = root.editMode
        preferences.editorPreferences.fontSize = root.fontSize
        preferences.editorPreferences.wordWrap = root.wordWrap
        LK.config.setPreferences(preferences)
    }

    function _setEditorKeybindings() {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            let handlerPath = null
            if (root.editMode === "vim") {
                handlerPath = "ace/keyboard/vim"
            } else if (root.editMode === "emacs") {
                handlerPath = "ace/keyboard/emacs"
            }

            root._aceEditorObject.callEditorFunction("setKeyboardHandler", handlerPath)
        }
    }

    function save() {
        if (root.commandId === "" || root.remoteFilePath === "") {
            root._vimCloseAfterSave = false
            return false
        }

        if (root.pendingInvocation !== 0) {
            if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
                root._aceEditorObject.showVimNotification(
                    "Save already in progress",
                    Theme.criticalityColor("Warning"))
            }
            return false
        }

        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.getContent(function(content) {
                if (root._vimCloseAfterSave) {
                    root._saveOverlayActive = true
                }
                root.saved(root.commandId, root.remoteFilePath, content)
            })
            return true
        } else if (root.textEditorItem) {
            if (root._vimCloseAfterSave) {
                root._saveOverlayActive = true
            }
            root.textEditorItem.save()
            return true
        }
        return false
    }

    function activate() {
        root._updateUseSimpleCodeEditor()
        
        if (root.pendingInvocation === 0) {
            if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
                root._aceEditorObject.getContent(function(content) {
                    root.contentChanged(root.remoteFilePath, content)
                    root.disableSaveButton = !LK.command.hasFileChanged(root.hostId, root.remoteFilePath, content)
                })
                Qt.callLater(function() {
                    if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
                        root._aceEditorObject.focusEditor()
                    }
                })
            } else if (root.textEditorItem) {
                root.textEditorItem.activate()
            }
        }
    }

    function deactivate() {
    }

    function refresh() {
    }

    function close() {
        root.closed(root.remoteFilePath)
    }
}
