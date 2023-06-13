import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

import ".."
import "../Text"


// This component should be a direct child of main window.
Dialog {
    id: root
    property string text: ""
    modal: true
    implicitWidth: 600
    implicitHeight: 400
    standardButtons: Dialog.Close

    background: Rectangle {
        color: Material.background
        border.width: 1
        border.color: "#808080"
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ScrollView {
        visible: root.text !== ""
        anchors.fill: parent

        NormalText {
            anchors.fill: parent
            wrapMode: Text.WordWrap
            textFormat: Text.MarkdownText
            text: root.text
        }
    }

    onClosed: {
        root.text = ""
    }
}