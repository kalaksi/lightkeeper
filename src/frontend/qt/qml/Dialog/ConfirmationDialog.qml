import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

import "../Text"


LightkeeperDialog {
    id: root

    property string text: ""

    title: "Confirmation"
    standardButtons: Dialog.Yes | Dialog.No
    implicitWidth: 350
    implicitHeight: 200
    anchors.centerIn: parent

    contentItem: Item {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog

        NormalText {
            text: root.text
            anchors.centerIn: parent
        }
    }

    Component.onCompleted: {
        visible = true
    }
}