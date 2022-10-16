import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

import "js/Parse.js" as Parse

Item {
    id: root
    property var text: ""
    property var errorText: ""
    property var criticality: ""

    Rectangle {
        color: Material.background
        anchors.fill: parent
    }

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