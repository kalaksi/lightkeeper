/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Text"
import "../Button"
import "../Misc"
import ".."
import "../StyleOverride"


LightkeeperDialog {
    id: root
    property string moduleId: ""
    property string groupName: ""
    property alias moduleSettings: repeater.model
    property bool _loading: moduleId === ""

    title: `Module settings: ${root.moduleId}`
    implicitWidth: 680
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal settingsUpdated(string moduleId, var settings)

    onAccepted: {
        let moduleSettings = []
        for (let i = 0; i < repeater.model.length; i++) {
            let nextItem = repeater.itemAt(i)
            // See `ModuleSetting` in ConfigManagerModel for the model.
            let moduleSetting = {
                "key": nextItem["_settingKey"],
                "value": nextItem["settingValue"](),
                "enabled": nextItem["_enabled"],
                "isSecret": nextItem["_isSecret"],
            }
            moduleSettings.push(moduleSetting)

            if (!moduleSetting.enabled && nextItem["_isSecret"] && root.groupName !== "") {
                LK.config.removeGroupSecret(root.groupName, root.moduleId, moduleSetting.key)
            }
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
                    property string _settingKey: modelData.key
                    property string _secretSaveValue: modelData.isSecret === true ? modelData.value : ""
                    property string _lastSecretBackend: ""
                    property string _effectiveSecretBackend: _lastSecretBackend !== ""
                        ? _lastSecretBackend : (modelData.secretBackend === "keyring" ? "keyring" : "plaintext")
                    property bool _enabled: toggleSwitch.checked
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
                            visible: modelData.isSecret !== true && !modelData.key.endsWith("_path")
                            enabled: toggleSwitch.checked
                            selectByMouse: true
                            placeholderText: toggleSwitch.checked ? "" : "unset"
                            placeholderTextColor: Theme.textColorDark
                            text: toggleSwitch.checked ? modelData.value : ""

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                        }

                        FilePathField {
                            id: filePathField
                            visible: modelData.isSecret !== true && modelData.key.endsWith("_path")
                            enabled: toggleSwitch.checked
                            placeholderText: toggleSwitch.checked ? "" : "unset"
                            placeholderTextColor: Theme.textColorDark
                            text: toggleSwitch.checked ? modelData.value : ""

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                        }

                        SecretValueField {
                            id: secretField
                            visible: modelData.isSecret === true
                            enabled: toggleSwitch.checked
                            settingKey: modelData.key
                            description: modelData.description ?? ""
                            saveValue: toggleSwitch.checked ? rowLayout._secretSaveValue : ""
                            backend: rowLayout._effectiveSecretBackend
                            onRevealRequested: secretField.revealSecret(rowLayout.secretValue())
                            onEditRequested: secretField.openEditor(rowLayout.secretValue())
                            onSecretSubmitted: function(value, backend) {
                                let wasSecretBackend = rowLayout._effectiveSecretBackend

                                if (root.groupName !== "") {
                                    if (value === "" || (backend === "plaintext" && wasSecretBackend === "keyring")) {
                                        LK.config.removeGroupSecret(root.groupName, root.moduleId, modelData.key)
                                    }
                                }

                                if (backend === "keyring" && value !== "") {
                                    let placeholder = LK.config.storeGroupSecret(
                                        root.groupName, root.moduleId, modelData.key, value)
                                    // Errors also result in empty string.
                                    if (placeholder !== "") {
                                        rowLayout._secretSaveValue = placeholder
                                    }
                                }
                                else {
                                    rowLayout._secretSaveValue = value
                                }
                                rowLayout._lastSecretBackend = backend
                            }

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                        }
                    }

                    function settingValue() {
                        if (rowLayout._isSecret) {
                            return rowLayout._secretSaveValue
                        }
                        if (modelData.key.endsWith("_path")) {
                            return filePathField.text
                        }
                        return textField.text
                    }

                    function secretValue() {
                        if (rowLayout._effectiveSecretBackend === "keyring" && root.groupName !== "") {
                            return LK.config.getGroupSecret(root.groupName, root.moduleId, modelData.key) || ""
                        }
                        return rowLayout._secretSaveValue !== "" ? rowLayout._secretSaveValue : (modelData.value ?? "")
                    }
                }
            }
        }
    }

    function resetModel() {
        root.moduleSettings = []
    }
}