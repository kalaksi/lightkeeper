import QtQuick
import QtQuick.Controls

import Theme
import Lighthouse.AceEditor 1.0

import ".."
import "../js/Utils.js" as Utils


Item {
    id: root
    required property string localFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0
    property string _detectedLanguage: Utils.detectLanguageFromPath(root.localFilePath)
    property string _aceMode: Utils.mapLanguageToAceMode(root._detectedLanguage)

    signal saved(commandId: string, localFilePath: string, content: string)
    signal closed(localFilePath: string)
    signal contentChanged(localFilePath: string, newContent: string)

    onLocalFilePathChanged: {
        root._detectedLanguage = Utils.detectLanguageFromPath(root.localFilePath)
        root._aceMode = Utils.mapLanguageToAceMode(root._detectedLanguage)
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

    AceEditor {
        id: aceEditor
        visible: root.text !== ""
        anchors.fill: parent
        anchors.margins: Theme.spacingLoose
        content: root.text
        mode: root._aceMode

        onEditorContentChanged: function(newContent) {
            root.contentChanged(root.localFilePath, newContent)
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

        aceEditor.editor.getContent(function(content) {
            root.saved(root.commandId, root.localFilePath, content)
        })
    }

    function activate() {
        if (root.pendingInvocation === 0) {
            aceEditor.editor.getContent(function(content) {
                root.contentChanged(root.localFilePath, content)
            })
        }
    }

    function deactivate() {
    }

    function refresh() {
    }

    function close() {
        root.closed(root.localFilePath)
    }

    onTextChanged: {
        aceEditor.content = root.text
    }

    on_AceModeChanged: {
        aceEditor.mode = root._aceMode
    }
}
