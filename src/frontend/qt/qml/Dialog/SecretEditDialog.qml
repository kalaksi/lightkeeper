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
    property string initialBackend: "keyring"
    property string initialValue: ""
    // Optional function(value) storing to keyring before closing. Returns placeholder or empty string on failure.
    property var storeSecret: null
    property bool _passwordVisible: false
    property string _errorText: ""

    property string selectedBackend: "keyring"

    title: "Manage secrets"
    modal: true
    implicitWidth: 560
    implicitHeight: rootColumn.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal secretSubmitted(string value, string backend, string placeholder)

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

        RowLayout {
            width: parent.width - parent.anchors.leftMargin - parent.anchors.rightMargin
            spacing: Theme.spacingNormal

            SmallText {
                text: "Secrets backend"
                color: Theme.textColorDark
                Layout.preferredWidth: 220
                Layout.alignment: Qt.AlignVCenter
            }

            ComboBox {
                id: backendComboBox
                model: ["keyring", "plaintext"]
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter
            }
        }

        NormalText {
            visible: root._errorText !== ""
            width: parent.width
            text: root._errorText
            color: Theme.colorForCriticality("Error")
            wrapMode: Text.WordWrap
        }
    }

    onOpened: {
        // Re-opened after a failed store: keep user's input.
        if (root._errorText !== "") {
            return
        }
        passwordField.text = root.initialValue
        root.selectedBackend = root.initialBackend
        backendComboBox.currentIndex = root.initialBackend === "plaintext" ? 1 : 0
    }

    onAccepted: {
        root.selectedBackend = backendComboBox.currentIndex === 1 ? "plaintext" : "keyring"
        let placeholder = ""
        if (root.selectedBackend === "keyring" && passwordField.text !== "" && root.storeSecret !== null) {
            placeholder = root.storeSecret(passwordField.text)
            if (placeholder === "") {
                root._errorText = "Failed to store secret in keyring. Check that the keyring / secret service is available."
                root.open()
                return
            }
        }
        root.secretSubmitted(passwordField.text, root.selectedBackend, placeholder)
        resetFields()
    }

    onRejected: {
        resetFields()
    }

    function resetFields() {
        passwordField.text = ""
        root._errorText = ""
        root._passwordVisible = false
        root.selectedBackend = "keyring"
        root.settingKey = ""
        root.description = ""
        root.initialBackend = "keyring"
        root.initialValue = ""
    }
}
