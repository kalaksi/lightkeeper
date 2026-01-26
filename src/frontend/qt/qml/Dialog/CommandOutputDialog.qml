/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../DetailsView"
import "../Text"
import "../js/TextTransform.js" as TextTransform


LightkeeperDialog {
    id: root
    property string text: ""
    property string errorText: ""
    property bool showProgress: true
    property int progress: 0
    property bool enableShortcuts: false
    property int pendingInvocation: 0

    modal: true
    implicitWidth: commandOutput.width + 100
    implicitHeight: commandOutput.height + 100
    standardButtons: Dialog.Close | Dialog.Reset

    signal moveToTab(int pendingInvocation, string title, string text, string errorText, int progress)

    Component.onCompleted: {
        root.standardButton(Dialog.Reset).text = "Move to a tab"
    }

    onClosed: {
        root.resetFields()
    }

    onReset: {
        root.moveToTab(root.pendingInvocation, root.title, root.text, root.errorText, root.progress)
        root.resetFields()
        root.close()
    }

    onTextChanged: {
        let coloredText = TextTransform.ansiToRichText(root.text)
        commandOutput.rows = coloredText.split("\n")

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
}