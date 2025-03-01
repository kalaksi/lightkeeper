import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../DetailsView"
import "../Text"
import "../Misc"
import "../js/TextTransform.js" as TextTransform


LightkeeperDialog {
    id: root
    property string text: ""
    property string errorText: ""
    property string commandText: ""
    property bool showProgress: true
    property int progress: 0
    property bool enableShortcuts: false

    modal: true
    implicitWidth: commandOutput.width + 100
    implicitHeight: commandOutput.height + 100
    standardButtons: Dialog.Close

    onClosed: {
        root.resetFields()
    }

    onTextChanged: {
        commandOutput.rows = root.text.split("\n")

        // Scroll to bottom.
        commandOutput.positionViewAtEnd()
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

                // The color can be wrong on some platforms and progress bar invisible, so force color.
                // This can also later be used to set color according to criticality level.
                contentItem: Rectangle {
                    implicitHeight: progressBar.height
                    implicitWidth: progressBar.width
                    color: "#202020"
                    radius: 4

                    Rectangle {
                        height: parent.height
                        width: progressBar.visualPosition * parent.width
                        color: palette.highlight
                        radius: parent.radius
                    }
                }
            }

            NormalText {
                id: label
                lineHeight: 0.9
                text: root.progress + " %"
            }
        }

        NormalText {
            id: commandText
            // TODO
            visible: false
            text: root.commandText
            height: root.commandText.length > 0 ? implicitHeight : 0
        }

        LogList {
            id: commandOutput
            rows: []
            enableShortcuts: root.enableShortcuts
            selectionColor: "transparent"
            appendOnly: true
            invertRowOrder: false

            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.rightMargin: Theme.marginScrollbar
        }

        NormalText {
            id: errorCode
            visible: false
            // TODO:
            // visible: root.errorText.length > 0
            height: root.errorText.length > 0 ? implicitHeight : 0
            text: root.errorText
            color: Theme.colorForCriticality("Error")
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
        root.errorText = ""
        root.commandText = ""
        root.progress = 0
        // In append-only mode, only resetting text is not enough.
        commandOutput.resetFields()
    }
}