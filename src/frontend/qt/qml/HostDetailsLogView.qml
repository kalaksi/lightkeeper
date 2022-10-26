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

    ScrollView {
        visible: root.text !== ""
        anchors.fill: parent

        NormalText {
            id: textContent
            wrapMode: Text.WordWrap
            text: root.text
        }
    }
}