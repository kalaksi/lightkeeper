/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Text"
import "../Button"
import "../StyleOverride"


LightkeeperDialog {
    id: root
    property string settingKey: ""
    property string description: ""
    property string initialValue: ""
    // Backend mismatch or store failure message (Error styling).
    property string warningText: ""
    property bool _passwordVisible: false

    title: "Manage secrets"
    modal: true
    implicitWidth: 560
    implicitHeight: rootColumn.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal secretSubmitted(string value)

    contentItem: Column {
        id: rootColumn
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        RowLayout {
            width: parent.width
            height: textContainer.implicitHeight
            spacing: Theme.spacingNormal

            Column {
                id: textContainer
                Layout.preferredWidth: 220
                Layout.alignment: Qt.AlignVCenter

                Label {
                    width: parent.width
                    text: root.settingKey
                }

                SmallText {
                    visible: root.description !== ""
                    width: parent.width
                    text: root.description
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.minimumWidth: 260
                Layout.alignment: Qt.AlignVCenter
                spacing: Theme.spacingNormal

                TextField {
                    id: passwordField
                    echoMode: root._passwordVisible ? TextInput.Normal : TextInput.Password
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignVCenter
                }

                ImageButton {
                    imageSource: "qrc:/main/images/button/view-visible"
                    size: passwordField.implicitHeight * 0.8
                    tooltip: root._passwordVisible ? "Hide password" : "Show password"
                    onClicked: {
                        root._passwordVisible = !root._passwordVisible
                    }

                    Layout.preferredWidth: passwordField.implicitHeight
                    Layout.alignment: Qt.AlignVCenter
                }
            }
        }

        SmallText {
            width: parent.width
            text: "Secrets use the backend configured in Preferences."
            color: Theme.textColorDark
            wrapMode: Text.WordWrap
        }

        NormalText {
            visible: root.warningText !== ""
            width: parent.width
            text: root.warningText
            color: Theme.colorForCriticality("Error")
            wrapMode: Text.WordWrap
        }
    }

    onOpened: {
        passwordField.text = root.initialValue
    }

    onAccepted: {
        let value = passwordField.text
        resetFields()
        root.secretSubmitted(value)
    }

    onRejected: {
        resetFields()
    }

    function resetFields() {
        passwordField.text = ""
        root.warningText = ""
        root._passwordVisible = false
        root.settingKey = ""
        root.description = ""
        root.initialValue = ""
    }
}
