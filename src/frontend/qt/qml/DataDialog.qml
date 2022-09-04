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

    property var message: ""
    property var error: ""
    property var criticality: ""

    WorkingSprite {
        id: loadingAnimation
        anchors.centerIn: parent
        visible: root.message === "" && root.error === ""
    }

    ScrollView {
        visible: root.message !== ""
        anchors.fill: parent

        JsonTextFormat {
            jsonText: root.message
        }
    }

    ErrorMessage {
        text: root.error
        criticality: root.criticality
        visible: root.error !== ""
    }

    function init() {
        root.open()
    }

    function update() {
        let data = root.model.get_command_data(root.model.get_selected_host())[0]
        if (typeof data !== "undefined") {
            let commandResult = JSON.parse(data)
            root.message = commandResult.message
            root.error = commandResult.error
            root.criticality = commandResult.criticality
        }
    }

}