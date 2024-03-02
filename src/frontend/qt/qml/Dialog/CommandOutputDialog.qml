import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../Text"
import "../Misc"


// This component should be a direct child of main window.
Dialog {
    id: root
    property string text: ""
    property bool showProgress: true
    property int progress: 0

    modal: true
    opacity: 0.0
    implicitWidth: dialogText.contentWidth + 100
    implicitHeight: dialogText.contentHeight + 100
    standardButtons: Dialog.Close

    background: DialogBackground { }

    onClosed: {
        root.text = ""
        root.progress = 0
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        Row {
            visible: root.showProgress
            spacing: Theme.spacingNormal

            Layout.fillWidth: true

            ProgressBar {
                id: progressBar
                width: parent.parent.width * 0.95
                value: root.progress / 100.0
            }

            NormalText {
                id: label
                lineHeight: 0.9
                text: root.progress + " %"
            }
        }
        

        ScrollView {
            id: scrollView
            visible: root.text !== ""
            contentWidth: parent.width

            Layout.fillHeight: true

            NormalText {
                id: dialogText
                wrapMode: Text.WrapAnywhere
                text: root.text
            }
        }

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