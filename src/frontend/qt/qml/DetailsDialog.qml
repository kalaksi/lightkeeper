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
    Component.onCompleted: visible = true

    property var text: ""
    property var errorText: ""
    property var criticality: ""

    WorkingSprite {
        visible: root.text === "" && root.errorText === ""
    }

    ScrollView {
        visible: root.text !== ""
        anchors.fill: parent

        JsonTextFormat {
            jsonText: root.text
        }
    }

    AlertMessage {
        text: root.errorText
        criticality: root.criticality
        visible: root.errorText !== ""
    }
}