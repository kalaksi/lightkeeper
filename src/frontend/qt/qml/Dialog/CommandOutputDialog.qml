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
            contentWidth: parent.width
            contentHeight: dialogText.contentHeight

            Layout.fillHeight: true
            Layout.rightMargin: Theme.marginScrollbar

            WorkingSprite {
                visible: root.text === ""
                height: parent.height
                width: parent.width
            }

            NormalText {
                id: dialogText
                visible: root.text !== ""
                wrapMode: Text.WrapAnywhere
                text: root.text

                onTextChanged: {
                    // Scroll to bottom
                    scrollView.ScrollBar.vertical.position = 1.0 - scrollView.ScrollBar.vertical.size
                }
            }
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

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animationDurationFast
        }
    }
}