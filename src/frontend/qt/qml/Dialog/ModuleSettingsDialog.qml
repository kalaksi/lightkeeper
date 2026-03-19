/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Text"
import "../Button"
import ".."
import "../StyleOverride"


LightkeeperDialog {
    id: root
    property string moduleId: ""
    property string groupName: ""
    property alias moduleSettings: repeater.model
    property bool _loading: moduleId === ""
    property var _secretTarget: null

    title: `Module settings: ${root.moduleId}`
    implicitWidth: 680
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal settingsUpdated(string moduleId, var settings)

    onAccepted: {
        let moduleSettings = []
        for (let i = 0; i < repeater.model.length; i++) {
            let nextItem = repeater.itemAt(i)
            let value = nextItem._secretSaveValue !== "" ? nextItem._secretSaveValue : nextItem.children[2].children[0].text
            // See `ModuleSetting` in ConfigManagerModel for the model.
            let moduleSetting = {
                "key": nextItem.children[0].children[0].text,
                "value": value,
                "enabled": nextItem.children[1].checked,
                "isSecret": nextItem._isSecret,
            }
            moduleSettings.push(moduleSetting)
        }

        root.settingsUpdated(root.moduleId, moduleSettings)
        root.resetModel()
    }

    onRejected: {
        root.resetModel()
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: scrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root._loading
    }

    contentItem: ScrollView {
        id: scrollView
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        contentWidth: availableWidth
        clip: true

        Column {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.marginScrollbar
            spacing: Theme.spacingNormal

            Repeater {
                id: repeater
                model: []

                RowLayout {
                    id: rowLayout
                    property string _secretSaveValue: modelData.isSecret === true ? modelData.value : ""
                    property string _lastSecretBackend: ""
                    property string _effectiveSecretBackend: _lastSecretBackend !== ""
                        ? _lastSecretBackend : (modelData.secretBackend === "keyring" ? "keyring" : "plaintext")
                    property string _revealedSecret: ""
                    property bool _isSecret: modelData.isSecret === true
                    width: parent.width
                    height: textContainer.implicitHeight
                    spacing: Theme.spacingNormal

                    Column {
                        id: textContainer
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter

                        Label {
                            width: parent.width
                            text: modelData.key
                        }

                        SmallText {
                            width: parent.width
                            text: modelData.description ?? ""
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }
                    }

                    Switch {
                        id: toggleSwitch
                        checked: modelData.enabled

                        Layout.alignment: Qt.AlignVCenter
                    }

                    RowLayout {
                        Layout.preferredWidth: scrollView.width * 0.35
                        Layout.alignment: Qt.AlignVCenter
                        spacing: Theme.spacingNormal

                        TextField {
                            id: textField
                            visible: !(modelData.isSecret === true && rowLayout._revealedSecret !== "")
                            enabled: toggleSwitch.checked && modelData.isSecret !== true
                            readOnly: modelData.isSecret === true
                            selectByMouse: modelData.isSecret !== true
                            placeholderText: toggleSwitch.checked ? "" : "unset"
                            placeholderTextColor: Theme.textColorDark
                            text: toggleSwitch.checked
                                ? (modelData.isSecret && rowLayout._secretSaveValue !== ""
                                    ? rowLayout._secretSaveValue : modelData.value)
                                : ""
                            echoMode: modelData.isSecret === true ? TextInput.Password : TextInput.Normal

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter

                            Connections {
                                target: DesktopPortal
                                function onFileChooserResponse(token, filePath) {
                                    if (fileChooserButton.visible && token === fileChooserButton._fileChooserToken) {
                                        textField.text = filePath
                                    }
                                }
                            }
                        }

                        TextField {
                            id: resolvedSecretField
                            visible: modelData.isSecret === true && rowLayout._revealedSecret !== ""
                            readOnly: true
                            selectByMouse: false
                            text: rowLayout._revealedSecret
                            placeholderText: ""

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                        }

                        ImageButton {
                            id: fileChooserButton
                            property string _fileChooserToken: ""

                            // TODO: this is quick and hacky, refactor.
                            visible: modelData.key.endsWith("_path")
                            enabled: toggleSwitch.checked
                            imageSource: "qrc:/main/images/button/document-open-folder"
                            size: textField.implicitHeight * 0.8
                            onClicked: {
                                _fileChooserToken = DesktopPortal.openFileChooser()
                            }

                            Layout.preferredWidth: textField.implicitHeight
                            Layout.alignment: Qt.AlignVCenter
                        }

                        ImageButton {
                            id: revealButton
                            visible: modelData.isSecret === true
                            enabled: toggleSwitch.checked
                            imageSource: "qrc:/main/images/button/view-visible"
                            size: textField.implicitHeight * 0.8
                            tooltip: rowLayout._revealedSecret !== "" ? "Hide password" : "Show password"
                            onClicked: {
                                if (rowLayout._revealedSecret !== "") {
                                    rowLayout._revealedSecret = ""
                                }
                                else {
                                    if (rowLayout._effectiveSecretBackend === "keyring" && root.groupName !== "") {
                                        let secret = LK.config.getGroupSecret(root.groupName, root.moduleId, modelData.key)
                                        rowLayout._revealedSecret = secret
                                    }
                                    else {
                                        rowLayout._revealedSecret = rowLayout._secretSaveValue !== ""
                                            ? rowLayout._secretSaveValue : (modelData.value ?? "")
                                    }
                                }
                            }

                            Layout.preferredWidth: textField.implicitHeight
                            Layout.alignment: Qt.AlignVCenter
                        }

                        ImageButton {
                            id: keyringButton
                            visible: modelData.isSecret === true && root.groupName !== ""
                            enabled: toggleSwitch.checked
                            imageSource: "qrc:/main/images/button/entry-edit"
                            size: textField.implicitHeight * 0.8
                            tooltip: "Edit secret"
                            onClicked: {
                                root._secretTarget = {
                                    row: rowLayout,
                                    key: modelData.key,
                                    wasSecretBackend: rowLayout._effectiveSecretBackend
                                }
                                secretDialog.settingKey = modelData.key
                                secretDialog.description = modelData.description ?? ""
                                secretDialog.showBackendSelector = true
                                secretDialog.initialBackend = rowLayout._effectiveSecretBackend
                                if (rowLayout._effectiveSecretBackend === "keyring" && root.groupName !== "") {
                                    secretDialog.initialValue = LK.config.getGroupSecret(root.groupName, root.moduleId, modelData.key) || ""
                                }
                                else {
                                    secretDialog.initialValue = textField.text
                                }
                                secretDialog.open()
                            }

                            Layout.preferredWidth: textField.implicitHeight
                            Layout.alignment: Qt.AlignVCenter
                        }
                    }

                }
            }
        }
    }

    function resetModel() {
        root.moduleSettings = []
    }

    SecretEditDialog {
        id: secretDialog

        onSecretSubmitted: function(value, backend) {
            if (root._secretTarget && value !== undefined) {
                let row = root._secretTarget.row
                let key = root._secretTarget.key
                let wasSecretBackend = root._secretTarget.wasSecretBackend
                root._secretTarget = null

                if (root.groupName !== "") {
                    if (value === "" || (backend === "plaintext" && wasSecretBackend === "keyring")) {
                        LK.config.removeGroupSecret(root.groupName, root.moduleId, key)
                    }
                }

                if (backend === "keyring" && value !== "") {
                    let placeholder = LK.config.storeGroupSecret(root.groupName, root.moduleId, key, value)
                    // Errors also result in empty string.
                    if (placeholder !== "") {
                        row._secretSaveValue = placeholder
                    }
                }
                else {
                    row._secretSaveValue = value
                }
                row._lastSecretBackend = backend
                row._revealedSecret = ""
            }
        }

        onRejected: {
            root._secretTarget = null
        }
    }
}