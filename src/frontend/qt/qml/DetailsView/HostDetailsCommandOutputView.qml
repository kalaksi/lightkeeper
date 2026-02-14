/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

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

                // Contains incremental update.
                let lastNewlineIndex = root.text.lastIndexOf("\n")
                root.text = lastNewlineIndex === -1 ?
                    commandResult.message :
                    root.text.substring(0, lastNewlineIndex + 1) + commandResult.message

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

        RowLayout {
            visible: root.showProgress
            spacing: Theme.spacingNormal

            Layout.fillWidth: true
            Layout.rightMargin: Theme.spacingLoose

            ProgressBar {
                id: progressBar
                implicitHeight: parent.height * 0.5
                value: root.progress / 100.0

                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

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

            Button {
                id: stopButton
                icon.source: "qrc:/main/images/button/stop"
                icon.height: 16
                icon.width: 16
                text: "Stop"
                enabled: root.pendingInvocation > 0 && root.progress < 100

                focusPolicy: Qt.NoFocus

                Layout.alignment: Qt.AlignVCenter

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Stop"

                onClicked: {
                    LK.command.interruptInvocation(root.pendingInvocation)
                }
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



