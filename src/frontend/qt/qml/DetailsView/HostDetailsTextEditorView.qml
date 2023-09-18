import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import ".."
import "../js/Parse.js" as Parse
import "../Text"

Item {
    id: root
    property string commandId: ""
    property var text: ""
    property string localFilePath: ""
    property var pendingInvocations: []


    signal saved(commandId: string, localFilePath: string, content: string)

    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocations.includes(commandResult.invocation_id)) {
                root.pendingInvocations = root.pendingInvocations.filter((invocationId) => invocationId != commandResult.invocationId)

                root.text = commandResult.message
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
            anchors.fill: parent
            wrapMode: Text.WordWrap
            color: Theme.color_text()
            text: root.text
            font.family: "monospace"
        }
    }

    function save() {
        if (root.commandId === "" || root.localFilePath === "") {
            return
        }

        let content = textEdit.text
        root.saved(root.commandId, root.localFilePath, content)
    }

    function open(commandId, invocationId, localFilePath) {
        reset()
        root.visible = true
        root.commandId = commandId
        root.localFilePath = localFilePath
        root.pendingInvocations.push(invocationId)
    }

    function reset() {
        root.text = ""
        root.commandId = ""
        root.localFilePath = ""
        root.pendingInvocations = []
    }
}