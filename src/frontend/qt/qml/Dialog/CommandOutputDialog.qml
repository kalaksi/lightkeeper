import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../DetailsView"
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
    implicitWidth: commandOutput.width + 100
    implicitHeight: commandOutput.height + 100
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

        LogList {
            id: commandOutput
            visible: root.text.length > 0
            rows: root.text.split("\n")
            enableShortcuts: true
            selectionColor: "transparent"
            invertRowOrder: false

            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.rightMargin: Theme.marginScrollbar
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