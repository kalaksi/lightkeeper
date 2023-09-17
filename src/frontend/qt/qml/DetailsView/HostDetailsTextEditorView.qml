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
    property var pendingInvocations: []


    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocations.includes(commandResult.invocation_id)) {
                root.pendingInvocations = root.pendingInvocations.filter((invocationId) => invocationId != commandResult.invocationId)

                root.text = commandResult.message
                root.errorText = commandResult.error
                root.criticality = commandResult.criticality
            }
        }
    }

    Rectangle {
        color: Theme.color_background()
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ScrollView {
        visible: root.text !== ""
        anchors.fill: parent

        TextEdit {
            anchors.fill: parent
            anchors.margins: 20
            wrapMode: Text.WordWrap
            color: Theme.color_text()
            text: root.text
        }
    }

    function open(invocationId) {
        root.visible = true
        reset()
        root.pendingInvocations.push(invocationId)
    }

    function reset() {
        root.text = ""
    }
}