import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

import ".."
import "../js/Parse.js" as Parse
import "../Text"

Item {
    id: root
    property var text: ""
    property var jsonText: ""
    property var errorText: ""
    property var criticality: ""
    property var pendingInvocations: []


    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocations.includes(commandResult.invocation_id)) {
                root.pendingInvocations = root.pendingInvocations.filter((invocationId) => invocationId != commandResult.invocationId)

                // If message seems to contain JSON...
                if (commandResult.message.startsWith("{")) {
                    root.jsonText = commandResult.message
                }
                else {
                    root.text = commandResult.message
                }

                root.errorText = commandResult.error
                root.criticality = commandResult.criticality
            }
        }
    }

    Rectangle {
        color: Material.background
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === "" && root.errorText === ""
    }

    ScrollView {
        visible: root.jsonText !== ""
        anchors.fill: parent

        JsonTextFormat {
            anchors.fill: parent
            anchors.margins: 20
            jsonText: root.jsonText
        }
    }

    ScrollView {
        visible: root.text !== ""
        anchors.fill: parent

        NormalText {
            anchors.fill: parent
            anchors.margins: 20
            wrapMode: Text.WordWrap
            textFormat: Text.MarkdownText
            text: root.text
        }
    }

    AlertMessage {
        text: root.errorText
        criticality: root.criticality
        visible: root.errorText !== ""
    }

    function open(commandId, invocationId) {
        reset()
        root.commandId = commandId
        root.pendingInvocations.push(invocationId)
    }

    function reset() {
        root.text = ""
        root.jsonText = ""
        root.errorText = ""
        root.criticality = ""
    }
}