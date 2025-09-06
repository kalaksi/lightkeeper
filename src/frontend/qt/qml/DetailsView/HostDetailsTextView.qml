import QtQuick
import QtQuick.Controls

import Theme

import ".."
import "../Text"

Item {
    id: root
    property var text: ""
    property var jsonText: ""
    property var errorText: ""
    property var criticality: ""
    property var pendingInvocation: -1


    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === invocationId) {
                root.pendingInvocations = -1

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
        color: Theme.backgroundColor
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

    AlertText {
        text: root.errorText
        criticality: root.criticality
        visible: root.errorText !== ""
    }

    function activate() {
        // Do nothing.
    }

    function deactivate() {
        // Do nothing.
    }

    function refresh() {
        // Do nothing.
    }

    function close() {
        // Do nothing.
    }
}