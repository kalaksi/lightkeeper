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
    property string hostId: ""
    property alias moduleSettings: repeater.model
    property bool _loading: moduleId === ""
    /// Host overrides: switch = override this key; off = inherit group value. Also scopes secrets to the host.
    property bool _hostScoped: root.hostId !== ""

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

            if (!moduleSetting.enabled && nextItem["_isSecret"]) {
                if (root._hostScoped) {
                    LK.config.removeHostSecret(root.hostId, root.moduleId, moduleSetting.key)
                }
                else if (root.groupName !== "") {
                    LK.config.removeGroupSecret(root.groupName, root.moduleId, moduleSetting.key)
                }
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
                    property string _inheritedValue: modelData.inheritedValue ?? ""
                    property bool _inheritedEnabled: modelData.inheritedEnabled === true
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
                        ToolTip.visible: root._hostScoped && hovered
                        ToolTip.delay: Theme.tooltipDelay
                        ToolTip.text: "Override on this host"

                        Layout.alignment: Qt.AlignVCenter

                        onToggled: {
                            if (!root._hostScoped) {
                                return
                            }
                            if (rowLayout._isSecret) {
                                if (toggleSwitch.checked
                                    && LK.config.detectSecretBackend(rowLayout._inheritedValue) === "keyring") {
                                    // Don't reuse a group keyring placeholder as a host override.
                                    rowLayout._secretSaveValue = ""
                                    rowLayout._lastSecretBackend = "plaintext"
                                }
                                else {
                                    rowLayout._secretSaveValue = rowLayout._inheritedValue
                                    if (!toggleSwitch.checked) {
                                        rowLayout._lastSecretBackend = ""
                                    }
                                }
                            }
                            else {
                                // Enabling: start from inherited. Disabling: show inherited again.
                                rowLayout.setFieldText(rowLayout._inheritedValue)
                            }
                        }
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
                            placeholderText: rowLayout.unsetPlaceholder()
                            placeholderTextColor: Theme.textColorDark
                            // Host mode: model value is override or inherited. Group mode: blank when unset.
                            text: toggleSwitch.checked || root._hostScoped ? modelData.value : ""

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                        }

                        FilePathField {
                            id: filePathField
                            visible: modelData.isSecret !== true && modelData.key.endsWith("_path")
                            enabled: toggleSwitch.checked
                            placeholderText: rowLayout.unsetPlaceholder()
                            placeholderTextColor: Theme.textColorDark
                            text: toggleSwitch.checked || root._hostScoped ? modelData.value : ""

                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                        }

                        SecretValueField {
                            id: secretField
                            visible: modelData.isSecret === true
                            enabled: toggleSwitch.checked
                            settingKey: modelData.key
                            description: modelData.description ?? ""
                            saveValue: toggleSwitch.checked || root._hostScoped
                                ? rowLayout._secretSaveValue
                                : ""
                            backend: rowLayout._effectiveSecretBackend
                            onRevealRequested: secretField.revealSecret(rowLayout.secretValue())
                            onEditRequested: secretField.openEditor(rowLayout.secretValue())
                            onSecretSubmitted: function(value, backend) {
                                let wasSecretBackend = rowLayout._effectiveSecretBackend

                                if (root._hostScoped) {
                                    if (value === "" || (backend === "plaintext" && wasSecretBackend === "keyring")) {
                                        LK.config.removeHostSecret(root.hostId, root.moduleId, modelData.key)
                                    }
                                }
                                else if (root.groupName !== "") {
                                    if (value === "" || (backend === "plaintext" && wasSecretBackend === "keyring")) {
                                        LK.config.removeGroupSecret(root.groupName, root.moduleId, modelData.key)
                                    }
                                }

                                if (backend === "keyring" && value !== "") {
                                    let placeholder = root._hostScoped
                                        ? LK.config.storeHostSecret(root.hostId, root.moduleId, modelData.key, value)
                                        : LK.config.storeGroupSecret(root.groupName, root.moduleId, modelData.key, value)
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

                    function unsetPlaceholder() {
                        if (toggleSwitch.checked) {
                            return ""
                        }
                        if (root._hostScoped) {
                            // Inherited empty string is a real value; only absent group keys are unset.
                            return rowLayout._inheritedEnabled ? "" : "(unset)"
                        }
                        return "(unset)"
                    }

                    function setFieldText(value) {
                        if (modelData.isSecret === true) {
                            return
                        }
                        if (modelData.key.endsWith("_path")) {
                            filePathField.text = value
                        }
                        else {
                            textField.text = value
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
                        // Inherited host row: do not look up host keyring (secret lives on the group).
                        if (!toggleSwitch.checked && root._hostScoped) {
                            return rowLayout._inheritedValue
                        }
                        if (rowLayout._effectiveSecretBackend === "keyring") {
                            if (root._hostScoped) {
                                return LK.config.getHostSecret(root.hostId, root.moduleId, modelData.key) || ""
                            }
                            if (root.groupName !== "") {
                                return LK.config.getGroupSecret(root.groupName, root.moduleId, modelData.key) || ""
                            }
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
