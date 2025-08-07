import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

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
    property var hostSettings: JSON.parse(LK.config.getHostSettings(hostId))
    property var _selectedGroups: LK.config.getSelectedGroups(hostId)
    property var _availableGroups: LK.config.getAvailableGroups(hostId)
    property int _contentWidth: 360
    property bool _loading: true
    title: "Host details"

    implicitWidth: 570
    implicitHeight: 700
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
        root._loading = false
        updateOkButton()
    }

    onAccepted: {
        // TODO: GUI for host settings (UseSudo etc.)
        let newSettings = {
            overrides: {
                connectors: {},
                host_settings: {}
            }
        }

        if (Utils.isIpv4OrIpv6Address(hostAddressField.text)) {
            newSettings.address = hostAddressField.text
        }
        else {
            newSettings.fqdn = hostAddressField.text
        }

        if (sshPortField.text !== "" && sshPortField.acceptableInput) {
            newSettings.overrides.connectors = {
                ssh: {
                    settings: {
                        port: sshPortField.text
                    }
                }
            }
        }

        if (useSudoCheckbox.checked) {
            newSettings.overrides.host_settings = ["use_sudo"]
        }

        LK.config.setHostSettings(root.hostId, hostIdField.text, JSON.stringify(newSettings))
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
        anchors.margins: 100
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        Column {
            spacing: Theme.spacingTight
            Layout.fillWidth: true

            Label {
                text: "Name"
            }

            TextField {
                id: hostIdField
                width: parent.width
                placeholderText: "Unique name for host..."
                placeholderTextColor: Theme.textColorDark
                text: root.hostId === "new-host-id" ? "" : root.hostId
                validator: RegularExpressionValidator {
                    regularExpression: /[a-zA-Z\d\-\.]+/
                }
                onTextChanged: root.updateOkButton()
            }
        }

        Column {
            spacing: Theme.spacingTight
            Layout.fillWidth: true

            Label {
                text: "IP Address or domain name"
            }

            TextField {
                id: hostAddressField
                width: parent.width
                placeholderText: ""
                placeholderTextColor: Theme.textColorDark
                text: root.hostSettings.address === undefined ? root.hostSettings.fqdn : root.hostSettings.address 
                validator: RegularExpressionValidator {
                    regularExpression: /[\.\:a-zA-Z\d\-]+/
                }
                onTextChanged: root.updateOkButton()
            }
        }

        Column {
            spacing: Theme.spacingTight
            Layout.fillWidth: true

            Label {
                text: "SSH port override"
            }

            SmallText {
                text: "Allows overriding on host-level.\nUsually port is configured in a group, e.g. in defaults-group."
                color: Theme.textColorDark
            }

            TextField {
                id: sshPortField
                width: parent.width
                text: {
                    if (root.hostSettings.overrides !== undefined &&
                        root.hostSettings.overrides.connectors !== undefined &&
                        root.hostSettings.overrides.connectors["ssh"] !== undefined &&
                        root.hostSettings.overrides.connectors["ssh"].settings["port"] !== undefined) {

                        return root.hostSettings.overrides.connectors["ssh"].settings["port"]
                    }
                    return ""
                }
                validator: RegularExpressionValidator {
                    regularExpression: /[1-9][0-9]{0,4}/
                }
                onTextChanged: root.updateOkButton()
            }
        }

        Row {
            Layout.fillWidth: true

            Switch {
                id: useSudoCheckbox
                enabled: false
                checked: true
                onCheckedChanged: root.updateOkButton()
            }

            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "Use sudo"
            }
        }

        // Just for extra spacing
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        BigText {
            text: "Configuration groups"

            Layout.alignment: Qt.AlignHCenter
        }

        RowLayout {
            SmallText {
                text: "Group order is significant.\nLater groups may override settings from earlier ones."
                color: Theme.textColorDark

                Layout.fillWidth: true
            }

            // Show effective configuration. Dimmed if mouse is not hovering.
            ImageButton {
                imageSource: "qrc:/main/images/button/search"
                size: root.buttonSize
                tooltip: "Show effective configuration"
                onClicked: {
                    let connectorsAndSettings = JSON.parse(LK.config.getEffectiveConnectorSettings(root.hostId, root._selectedGroups))
                    let monitorsAndSettings = JSON.parse(LK.config.getEffectiveMonitorSettings(root.hostId, root._selectedGroups))
                    let commandsAndSettings = JSON.parse(LK.config.getEffectiveCommandSettings(root.hostId, root._selectedGroups))
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
                                    validator_regexp: "[a-zA-Z\d\-]+",
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
        }    }

    GroupConfigurationDialog {
        id: groupConfigDialog
        groupName: selectedGroupsList.currentIndex === -1 ? "" : root._selectedGroups[selectedGroupsList.currentIndex]

        // Not used currently but could prove useful later.
        // contentItem's margin seems to affect this dialog's position, so compensating for it here.
        // x: -100 + Theme.marginDialog
        // anchors.centerIn: undefined
    }

    GroupConfigurationDialog {
        id: effectiveConfigDialog
        readOnly: true
        groupName: ""
        title: "Effective configuration"

        // Not used currently but could prove useful later.
        // contentItem's margin seems to affect this dialog's position, so compensating for it here.
        // x: -100 + Theme.marginDialog
        // anchors.centerIn: undefined
    }

    InputDialog {
        id: groupAddDialog
        width: parent.width
        height: 150

        inputSpecs: [{
            label: "Configuration group name",
            field_type: "Text",
            validator_regexp: "[a-zA-Z\d\-]+",
        }]

        onInputValuesGiven: function(inputValues) {
            LK.config.addGroup(inputValues[0])
            LK.config.writeGroupConfiguration()
            root._availableGroups = root._availableGroups.concat(inputValues[0]).sort()
        }
    }


    function fieldsAreValid() {
        return hostIdField.acceptableInput &&
               hostAddressField.acceptableInput
    }

    function updateOkButton() {
        root.standardButton(Dialog.Ok).enabled = fieldsAreValid()
    }
}