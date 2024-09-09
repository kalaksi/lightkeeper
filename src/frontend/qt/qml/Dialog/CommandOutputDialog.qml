import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../DetailsView"
import "../Text"
import "../Misc"
import "../js/TextTransform.js" as TextTransform


// This component should be a direct child of main window.
LightkeeperDialog {
    id: root
    property string text: ""
    property bool showProgress: true
    property int progress: 0
    property bool enableShortcuts: false

    modal: true
    implicitWidth: commandOutput.width + 100
    implicitHeight: commandOutput.height + 100
    standardButtons: Dialog.Close

    onClosed: resetFields()

    onTextChanged: {
        if (root.text.length > 0) {
            commandOutput.rows = root.text.split("\n")
        }
        else {
            commandOutput.resetFields()
        }
    }


    contentItem: ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        Row {
            visible: root.showProgress
            spacing: Theme.spacingNormal

            Layout.fillWidth: true

            ProgressBar {
                id: progressBar
                anchors.verticalCenter: parent.verticalCenter
                width: parent.parent.width * 0.95
                height: parent.height * 0.5
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
            rows: []
            enableShortcuts: root.enableShortcuts
            selectionColor: "transparent"
            appendOnly: true
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

    function resetFields() {
        root.text = ""
        root.progress = 0
    }
}