import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

import "../Text"
import "../js/Utils.js" as Utils


LightkeeperDialog {
    id: root

    property string text: ""
    // Center short text automatically.
    property bool centerText: text.length < 40

    title: "Confirmation"
    standardButtons: Dialog.Yes | Dialog.No
    implicitWidth: Utils.clamp(dialogText.implicitWidth, 300, 800)
    implicitHeight: Utils.clamp(dialogText.implicitHeight, 200, 600)
    anchors.centerIn: parent

    contentItem: Item {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom

        NormalText {
            id: dialogText
            text: root.text
            width: parent.width
            wrapMode: Text.Wrap
            horizontalAlignment: root.centerText ? Text.AlignHCenter : Text.AlignLeft
        }
    }

    Component.onCompleted: {
        visible = true
    }
}