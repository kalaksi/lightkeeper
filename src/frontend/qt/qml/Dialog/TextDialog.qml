import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../Text"


// This component should be a direct child of main window.
Dialog {
    id: root
    property string text: ""
    property alias textFormat: dialogText.textFormat
    property alias wrapMode: dialogText.wrapMode

    modal: true
    opacity: 0.0
    // TODO: more accurate calculation?
    implicitWidth: dialogText.contentWidth + 100
    implicitHeight: dialogText.contentHeight + 100
    standardButtons: Dialog.Close

    background: DialogBackground { }


    WorkingSprite {
        visible: root.text === ""
    }

    ScrollView {
        id: scrollView
        visible: root.text !== ""
        anchors.fill: parent

        NormalText {
            id: dialogText
            wrapMode: Text.WordWrap
            textFormat: Text.MarkdownText
            text: root.text
        }
    }

    onClosed: {
        root.text = ""
    }

    Behavior on width {
        NumberAnimation {
            duration: Theme.animationDuration
        }
    }

    Behavior on height {
        NumberAnimation {
            duration: Theme.animationDuration
        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animationDuration
        }
    }
}