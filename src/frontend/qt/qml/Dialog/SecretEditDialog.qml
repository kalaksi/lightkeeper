/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Text"
import "../Button"
import "../StyleOverride"


LightkeeperDialog {
    id: root
    property string settingKey: ""
    property string description: ""
    property bool showBackendSelector: false
    property string initialBackend: "keyring"
    property string initialValue: ""
    property bool _passwordVisible: false

    property string selectedBackend: "keyring"

    title: "Manage secrets"
    modal: true
    implicitWidth: 560
    implicitHeight: rootColumn.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal secretSubmitted(string value, string backend)

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
            visible: root.showBackendSelector
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
    }

    onOpened: {
        passwordField.text = root.initialValue
        root.selectedBackend = root.initialBackend
        backendComboBox.currentIndex = root.initialBackend === "plaintext" ? 1 : 0
    }

    onAccepted: {
        root.selectedBackend = backendComboBox.currentIndex === 1 ? "plaintext" : "keyring"
        root.secretSubmitted(passwordField.text, root.selectedBackend)
        resetFields()
    }

    onRejected: {
        resetFields()
    }

    function resetFields() {
        passwordField.text = ""
        root._passwordVisible = false
        root.selectedBackend = "keyring"
        root.settingKey = ""
        root.description = ""
        root.showBackendSelector = false
        root.initialBackend = "keyring"
        root.initialValue = ""
    }
}
