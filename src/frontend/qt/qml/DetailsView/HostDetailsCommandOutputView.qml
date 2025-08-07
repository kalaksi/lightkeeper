import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Text"

/// Content mainly copied from CommandOutputDialog.qml, keep in sync. Split into separate component if needed.
Item {
    id: root
    property string text: ""
    property string errorText: ""
    property bool showProgress: true
    property int progress: 0
    property bool enableShortcuts: false
    property int pendingInvocation: 0

    onTextChanged: {
        commandOutput.rows = root.text.split("\n")

        // Scroll to bottom.
        commandOutput.positionViewAtEnd()
    }

    Component.onCompleted: {
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            if (root.pendingInvocation === invocationId) {
                let commandResult = JSON.parse(commandResultJson)

                root.text = commandResult.message
                root.errorText = commandResult.error
                root.progress = commandResult.progress
            }
        }
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    ColumnLayout {
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
            text: root.errorText
            color: Theme.colorForCriticality("Error")

            Layout.preferredHeight: root.errorText.length > 0 ? implicitHeight : 0
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
        root.progress = 0
        root.pendingInvocation = 0
        // In append-only mode, only resetting text is not enough.
        commandOutput.resetFields()
    }

    function close() {
        root.resetFields()
    }

    function activate() {
        root.enableShortcuts = true
    }

    function deactivate() {
        root.enableShortcuts = false
    }

    function refresh()  {
        // Do nothing.
    }
}



