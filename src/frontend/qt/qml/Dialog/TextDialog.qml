import QtQuick
import QtQuick.Controls

import Theme

import ".."
import "../Text"
import "../StyleOverride"


LightkeeperDialog {
    id: root
    property string text: ""
    property alias textFormat: dialogText.textFormat
    property alias wrapMode: dialogText.wrapMode

    modal: true
    implicitWidth: dialogText.contentWidth + 100
    implicitHeight: dialogText.contentHeight + 100
    standardButtons: Dialog.Close

    onClosed: {
        root.text = ""
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: scrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ScrollView {
        id: scrollView
        visible: root.text !== ""
        anchors.fill: parent
        anchors.margins: Theme.marginDialog

        NormalText {
            id: dialogText
            anchors.fill: parent
            wrapMode: Text.WordWrap
            textFormat: Text.MarkdownText
            text: root.text
        }
    }

    Behavior on width {
        NumberAnimation {
            duration: Theme.animationDurationFast
        }
    }

    Behavior on height {
        NumberAnimation {
            duration: Theme.animationDurationFast
        }
    }
}