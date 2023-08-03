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
    opacity: 0.0
    // TODO: more accurate calculation?
    implicitWidth: dialogText.implicitWidth + 100
    implicitHeight: dialogText.implicitHeight + 100
    standardButtons: Dialog.Close

    background: DialogBackground { }


    onVisibleChanged: {
        if (visible) {
            root.opacity = 1.0
        }
        else {
            root.opacity = 0.0
        }
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ScrollView {
        id: scrollView
        visible: root.text !== ""
        anchors.fill: parent

        NormalText {
            id: dialogText
            anchors.fill: parent
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
            duration: Theme.animation_duration()
        }
    }

    Behavior on height {
        NumberAnimation {
            duration: Theme.animation_duration()
        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animation_duration()
        }
    }
}