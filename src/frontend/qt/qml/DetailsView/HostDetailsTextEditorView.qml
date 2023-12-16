import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import ".."
import "../js/Parse.js" as Parse
import "../Text"

Item {
    id: root
    required property string localFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0


    signal saved(commandId: string, localFilePath: string, content: string)
    signal closed(localFilePath: string)
    signal contentChanged(localFilePath: string, newContent: string)

    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === commandResult.invocation_id) {
                root.pendingInvocation = 0

                if (commandResult.criticality === "Normal") {
                    root.text = commandResult.message
                }

                root.focus()
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
        anchors.margins: Theme.spacing_loose()
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

            onTextChanged: root.focus()
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

    function focus() {
        // If still waiting for data, then content can't have changed yet.
        if (root.pendingInvocation === 0) {
            // Update save-button enabled-status.
            root.contentChanged(root.localFilePath, textEdit.text)
        }
    }

    function unfocus() {
        // Do nothing.
    }

    function refresh() {
        // DO nothing.
    }

    function close() {
        root.closed(root.localFilePath)
    }
}