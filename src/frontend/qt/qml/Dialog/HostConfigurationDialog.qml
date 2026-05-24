/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Button"
import "../Text"
import "../Misc"
import "../js/Utils.js" as Utils
import ".."
import "../StyleOverride"

LightkeeperDialog {
    id: root
    property string hostId: ""
    property int buttonSize: 38
    property var hostSettings: ({})
    property var _selectedGroups: []
    property var _availableGroups: []
    property int _contentWidth: 380
    property int _formLeftColumnWidth: 320
    property int _formRightColumnWidth: 128
    property bool _loading: true
    // Refreshed on every dialog open so reopens pick up the latest config.
    property var _effectiveSettings: ({})
    property var _sshModuleSettings: []
    title: "Host details"

    implicitWidth: 660
    implicitHeight: 800
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged()

    onOpened: {
        LK.config.beginHostConfiguration()
        if (root.hostId === "") {
            LK.config.addHost("new-host-id")
            root.hostId = "new-host-id"
        }
        root._selectedGroups = LK.config.getSelectedGroups(root.hostId)
        root._availableGroups = LK.config.getAvailableGroups(root.hostId)
        root.hostSettings = JSON.parse(LK.config.getHostSettings(root.hostId))
        root._effectiveSettings = JSON.parse(
            LK.config.getEffectiveModuleSettings(root.hostId, root._selectedGroups, "connector"))
        root._sshModuleSettings = LK.config.getHostConnectorModuleSettings(root.hostId, "ssh").map(JSON.parse)
        root._loading = false
        updateOkButton()
    }

    onAccepted: {
        let newSettings = {
            overrides: {
                connectors: {},
                host_settings: []
            }
        }

        if (Utils.isIpv4OrIpv6Address(hostAddressField.text)) {
            newSettings.address = hostAddressField.text
        }
        else {
            newSettings.fqdn = hostAddressField.text
        }

        if (useSudoCheckbox.checked) {
            newSettings.overrides.host_settings = ["use_sudo"]
        }
        else {
            newSettings.overrides.host_settings = []
        }

        let sshSettings = {}
        for (let setting of root._sshModuleSettings) {
            if (setting.enabled) {
                sshSettings[setting.key] = setting.value
            }
        }
        for (let key of sshAuthOverride.ownedSshKeys) {
            delete sshSettings[key]
        }
        Object.assign(sshSettings, sshAuthOverride.buildAuthFields())

        if (sshPortField.text !== "" && sshPortField.acceptableInput) {
            sshSettings.port = sshPortField.text
        }
        else {
            delete sshSettings.port
        }

        // First write: ensure host gets renamed (so commitSecrets uses the final id).
        // Secrets/placeholders are filled in by commitSecrets and a second write follows.
        if (Object.keys(sshSettings).length > 0) {
            newSettings.overrides.connectors = { ssh: { settings: sshSettings } }
        }
        LK.config.setHostSettings(root.hostId, hostIdField.text, JSON.stringify(newSettings))

        sshAuthOverride.commitSecrets(hostIdField.text, sshSettings)

        if (Object.keys(sshSettings).length > 0) {
            newSettings.overrides.connectors = { ssh: { settings: sshSettings } }
        }
        else {
            newSettings.overrides.connectors = {}
        }
        LK.config.setHostSettings(hostIdField.text, hostIdField.text, JSON.stringify(newSettings))

        LK.config.updateHostGroups(hostIdField.text, root._selectedGroups)
        LK.config.endHostConfiguration()
        root._loading = true

        root.configurationChanged()
    }

    onRejected: {
        LK.config.cancelHostConfiguration()
        root._loading = true
    }

    Item {
        visible: root._loading
        Layout.fillWidth: true
        Layout.fillHeight: true

        WorkingSprite {
        }
    }

    contentItem: ColumnLayout {
        id: content
        visible: !root._loading
        anchors.fill: parent
        anchors.leftMargin: 90
        anchors.rightMargin: 90
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            rowSpacing: Theme.spacingLoose
            columnSpacing: Theme.spacingLoose

            ColumnLayout {
                spacing: Theme.spacingTight

                Layout.row: 0
                Layout.column: 0
                Layout.preferredWidth: root._formLeftColumnWidth
                Layout.maximumWidth: root._formLeftColumnWidth
                Layout.alignment: Qt.AlignBottom

                Label {
                    text: "Name"
                }

                TextField {
                    id: hostIdField
                    Layout.fillWidth: true
                    placeholderText: "Unique name for host..."
                    placeholderTextColor: Theme.textColorDark
                    text: root.hostId === "new-host-id" ? "" : root.hostId
                    readOnly: root.hostId !== "" && root.hostId !== "new-host-id"
                    validator: RegularExpressionValidator {
                        regularExpression: /[a-zA-Z\d\-\.]+/
                    }
                    onTextChanged: root.updateOkButton()
                }
            }

            ColumnLayout {
                spacing: Theme.spacingTight

                Layout.row: 0
                Layout.column: 1
                Layout.preferredWidth: root._formRightColumnWidth
                Layout.minimumWidth: root._formRightColumnWidth
                Layout.maximumWidth: root._formRightColumnWidth
                Layout.alignment: Qt.AlignBottom

                Label {
                    Layout.fillWidth: true
                    horizontalAlignment: Text.AlignHCenter
                    text: "Allow sudo"
                }

                Switch {
                    id: useSudoCheckbox
                    Layout.alignment: Qt.AlignHCenter
                    checked: {
                        if (root.hostSettings.overrides !== undefined &&
                            root.hostSettings.overrides.host_settings !== undefined) {
                            return root.hostSettings.overrides.host_settings.indexOf("use_sudo") !== -1
                        }
                        if (root.hostSettings.effective !== undefined &&
                            root.hostSettings.effective.host_settings !== undefined) {
                            return root.hostSettings.effective.host_settings.indexOf("use_sudo") !== -1
                        }
                        return true
                    }
                    onCheckedChanged: root.updateOkButton()
                }
            }

            ColumnLayout {
                spacing: Theme.spacingTight

                Layout.row: 1
                Layout.column: 0
                Layout.preferredWidth: root._formLeftColumnWidth
                Layout.maximumWidth: root._formLeftColumnWidth
                Layout.alignment: Qt.AlignBottom

                Label {
                    text: "Domain name"
                }

                SmallText {
                    Layout.fillWidth: true
                    text: "or IP address"
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }

                TextField {
                    id: hostAddressField
                    Layout.fillWidth: true
                    placeholderText: ""
                    placeholderTextColor: Theme.textColorDark
                    text: root.hostSettings.address === undefined ?
                        (root.hostSettings.fqdn ?? "") :
                        (root.hostSettings.address ?? "")
                    validator: RegularExpressionValidator {
                        regularExpression: /[\.\:a-zA-Z\d\-]+/
                    }
                    onTextChanged: root.updateOkButton()
                }
            }

            ColumnLayout {
                spacing: Theme.spacingTight

                Layout.row: 1
                Layout.column: 1
                Layout.preferredWidth: root._formRightColumnWidth
                Layout.minimumWidth: root._formRightColumnWidth
                Layout.maximumWidth: root._formRightColumnWidth
                Layout.alignment: Qt.AlignBottom

                Label {
                    Layout.fillWidth: true
                    text: "SSH port override"
                }

                SmallText {
                    Layout.fillWidth: true
                    text: "Host-level override"
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }

                TextField {
                    id: sshPortField
                    Layout.fillWidth: true
                    placeholderText: root.effectiveSshSetting("port")
                    placeholderTextColor: Theme.textColorDark
                    text: {
                        let port = root._sshModuleSettings.find(setting => setting.key === "port")
                        return port?.enabled ? port.value : ""
                    }
                    validator: RegularExpressionValidator {
                        regularExpression: /[1-9][0-9]{0,4}/
                    }
                    onTextChanged: root.updateOkButton()
                }
            }
        }

        SshAuthOverride {
            id: sshAuthOverride
            hostId: root.hostId
            inheritedUsername: root.effectiveSshSetting("username")
            moduleSettings: root._sshModuleSettings.filter(
                setting => sshAuthOverride.ownedSshKeys.indexOf(setting.key) !== -1)

            Layout.fillWidth: true
        }

        BigText {
            text: "Configuration groups"

            Layout.alignment: Qt.AlignHCenter
        }

        RowLayout {
            SmallText {
                text: "Config groups should provide bulk of the configuration.\nOrder is significant. Later groups can override settings from earlier ones."
                color: Theme.textColorDark

                Layout.fillWidth: true
            }

            // Show effective configuration. Dimmed if mouse is not hovering.
            ImageButton {
                imageSource: "qrc:/main/images/button/search"
                size: root.buttonSize
                tooltip: "Show effective configuration"
                onClicked: {
                    let connectorsAndSettings = JSON.parse(LK.config.getEffectiveModuleSettings(root.hostId, root._selectedGroups, "connector"))
                    let monitorsAndSettings = JSON.parse(LK.config.getEffectiveModuleSettings(root.hostId, root._selectedGroups, "monitor"))
                    let commandsAndSettings = JSON.parse(LK.config.getEffectiveModuleSettings(root.hostId, root._selectedGroups, "command"))
                    effectiveConfigDialog.groupConnectorSettings = connectorsAndSettings
                    effectiveConfigDialog.groupMonitorSettings = monitorsAndSettings
                    effectiveConfigDialog.groupCommandSettings = commandsAndSettings
                    effectiveConfigDialog.open()
                }
            }
        }

        ColumnLayout {
            spacing: 0

            Layout.fillHeight: true
            Layout.fillWidth: true

            TabBar {
                id: tabBar
                currentIndex: 0
                contentHeight: 36
                rightPadding: root.buttonSize + 4

                Layout.fillWidth: true

                LKTabButton {
                    text: `Selected (${root._selectedGroups.length})`
                    active: tabBar.currentIndex === 0
                }

                LKTabButton {
                    text: `Available (${root._availableGroups.length})`
                    active: tabBar.currentIndex === 1
                }
            }

            StackLayout {
                id: tabStackLayout
                currentIndex: tabBar.currentIndex

                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.minimumHeight: 220

                RowLayout {
                    Layout.fillWidth: true
                    Layout.fillHeight: true


                    LKListView {
                        id: selectedGroupsList
                        model: root._selectedGroups

                        Layout.fillHeight: true
                        Layout.fillWidth: true
                    }

                    // Remove, configure and reordering buttons.
                    ColumnLayout {
                        spacing: Theme.spacingNormal
                        layoutDirection: Qt.LeftToRight

                        Layout.preferredWidth: root.buttonSize
                        Layout.fillHeight: true

                        ImageButton {
                            id: removeButton
                            enabled: selectedGroupsList.currentIndex !== -1
                            imageSource: "qrc:/main/images/button/remove"
                            size: root.buttonSize
                            onClicked: {
                                let selectedGroup = root._selectedGroups[selectedGroupsList.currentIndex]
                                root._selectedGroups = root._selectedGroups.filter(group => group !== selectedGroup)
                                root._availableGroups = root._availableGroups.concat(selectedGroup).sort()
                            }
                        }

                        ImageButton {
                            enabled: selectedGroupsList.currentIndex !== -1
                            imageSource: "qrc:/main/images/button/configure"
                            size: root.buttonSize
                            onClicked: {
                                groupConfigDialog.groupName = root._selectedGroups[selectedGroupsList.currentIndex]
                                groupConfigDialog.open()
                            }
                        }

                        // Spacer
                        Item {
                            Layout.fillHeight: true
                        }

                        ImageButton {
                            id: moveUpButton
                            enabled: selectedGroupsList.currentIndex > 0
                            imageSource: "qrc:/main/images/button/arrow-up"
                            size: root.buttonSize
                            onClicked: {
                                // Use a copy to trigger re-rendering.
                                let result = root._selectedGroups.slice()
                                let previousIndex = selectedGroupsList.currentIndex
                                let selectedGroup = result[selectedGroupsList.currentIndex]

                                result.splice(selectedGroupsList.currentIndex, 1)
                                result.splice(selectedGroupsList.currentIndex - 1, 0, selectedGroup)
                                root._selectedGroups = result

                                selectedGroupsList.currentIndex = previousIndex - 1
                            }
                        }

                        ImageButton {
                            id: moveDownButton
                            enabled: selectedGroupsList.currentIndex < root._selectedGroups.length - 1
                            imageSource: "qrc:/main/images/button/arrow-down"
                            size: root.buttonSize
                            onClicked: {
                                // Use a copy to trigger re-rendering.
                                let result = root._selectedGroups.slice()
                                let previousIndex = selectedGroupsList.currentIndex
                                let selectedGroup = result[selectedGroupsList.currentIndex]

                                result.splice(selectedGroupsList.currentIndex, 1)
                                result.splice(selectedGroupsList.currentIndex + 1, 0, selectedGroup)
                                root._selectedGroups = result

                                selectedGroupsList.currentIndex = previousIndex + 1
                            }
                        }
                    }
                }

                RowLayout {
                    LKListView {
                        id: availableGroupsList
                        model: root._availableGroups

                        Layout.fillHeight: true
                        Layout.fillWidth: true
                    }

                    ColumnLayout {
                        spacing: Theme.spacingNormal

                        Layout.preferredWidth: root.buttonSize
                        Layout.fillHeight: true

                        ImageButton {
                            id: addButton
                            enabled: availableGroupsList.currentIndex !== -1
                            imageSource: "qrc:/main/images/button/add"
                            size: root.buttonSize
                            onClicked: {
                                let selectedGroup = root._availableGroups[availableGroupsList.currentIndex]
                                root._selectedGroups = root._selectedGroups.concat(selectedGroup)
                                root._availableGroups = root._availableGroups.filter(group => group !== selectedGroup)
                            }
                        }

                        ImageButton {
                            enabled: availableGroupsList.currentIndex !== -1
                            imageSource: "qrc:/main/images/button/configure"
                            size: root.buttonSize
                            onClicked: {
                                let selectedGroup = root._availableGroups[availableGroupsList.currentIndex]
                                groupConfigDialog.groupName = selectedGroup
                                groupConfigDialog.open()
                            }
                        }

                        // Spacer
                        Item {
                            Layout.fillHeight: true
                        }

                        ImageButton {
                            id: createGroupButton
                            imageSource: "qrc:/main/images/button/group-new"
                            tooltip: "Create new group"
                            size: root.buttonSize

                            onClicked: {
                                groupAddDialog.inputSpecs = [{
                                    label: "Configuration group name",
                                    field_type: "Text",
                                    validator_regexp: "[a-zA-Z0-9\\-]+",
                                }]
                                groupAddDialog.open()
                            }
                        }

                        ImageButton {
                            id: deleteGroupButton
                            imageSource: "qrc:/main/images/button/delete"
                            tooltip: "Delete group"
                            size: root.buttonSize

                            onClicked: {
                                let selectedGroup = root._availableGroups[availableGroupsList.currentIndex]
                                LK.config.removeGroup(selectedGroup)
                                LK.config.writeGroupConfiguration()
                                root._availableGroups = root._availableGroups.filter(group => group !== selectedGroup)
                            }
                        }
                    }
                }
            }
        }
    }

    GroupConfigurationDialog {
        id: groupConfigDialog
        groupName: selectedGroupsList.currentIndex === -1 ? "" : root._selectedGroups[selectedGroupsList.currentIndex]
        topMargin: 100

        // TODO
        // This doesn't work correctly on flatpak, maybe different Qt version?
        // Leaving it centered. It's less pretty but works more reliably for now.
        // anchors.centerIn: undefined
        // x: root.x + Theme.marginDialog
        // y: root.y + Theme.marginDialogTop
    }

    GroupConfigurationDialog {
        id: effectiveConfigDialog
        readOnly: true
        groupName: ""
        title: "Effective configuration"

        // TODO
        // This doesn't work correctly on flatpak, maybe different Qt version?
        // Leaving it centered. It's less pretty but works more reliably for now.
        // anchors.centerIn: undefined
        // x: root.x + Theme.marginDialog
        // y: root.y + Theme.marginDialogTop
    }

    InputDialog {
        id: groupAddDialog
        width: 500
        height: 150

        inputSpecs: [{
            label: "Configuration group name",
            field_type: "Text",
            validator_regexp: "[a-zA-Z0-9\\-]+",
        }]

        onInputValuesGiven: function(inputValues) {
            LK.config.addGroup(inputValues[0])
            LK.config.writeGroupConfiguration()
            root._availableGroups = root._availableGroups.concat(inputValues[0]).sort()
        }
    }


    function effectiveSshSetting(key) {
        return root._effectiveSettings["ssh"]
            ?.find(setting => setting.key === key)
            ?.value ?? ""
    }

    function updateOkButton() {
        let fieldsAreValid = hostIdField.acceptableInput && hostAddressField.acceptableInput
        root.standardButton(Dialog.Ok).enabled = fieldsAreValid
    }
}