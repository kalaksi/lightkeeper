import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import "js/Parse.js" as Parse

Dialog {
    id: root
    implicitHeight: parent.height
    implicitWidth: parent.width
    modal: false
    standardButtons: Dialog.Ok

    required property var model
    property var commandMessage: ""


    WorkingSprite {
        id: loadingAnimation
        anchors.centerIn: parent
        visible: root.commandMessage === ""
    }

    ScrollView {
        visible: root.commandMessage !== ""
        anchors.fill: parent

        JsonTextFormat {
            id: content
            jsonText: root.commandMessage
        }
    }

    function init() {
        root.open()
    }

    function update() {
        let commandResult = root.model.get_command_data(root.model.get_selected_host())[0]
        if (typeof commandResult !== "undefined") {
            root.commandMessage = JSON.parse(commandResult).message
        }
    }

}