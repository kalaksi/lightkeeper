import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../StyleOverride"
import "../Button"
import "../Text"
import "../Misc"
import "../js/Utils.js" as Utils
import ".."

// This component should be a direct child of main window.
LightkeeperDialog {
    id: root
    property string hostId: ""
    property int buttonSize: 38
    property var hostSettings: JSON.parse(ConfigManager.getHostSettings(hostId))
    property var _selectedGroups: ConfigManager.getSelectedGroups(hostId)
    property var _availableGroups: ConfigManager.getAvailableGroups(hostId)
    property int _contentWidth: 360
    property bool _loading: true
    title: "Host details"

    implicitWidth: 550
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged()

    onOpened: {
        ConfigManager.begin_host_configuration()
        if (root.hostId === "") {
            ConfigManager.add_host("new-host-id")
            root.hostId = "new-host-id"
        }
        refreshGroups()
        updateOkButton()
        root._loading = false
    }

    onAccepted: {
        // TODO: GUI for host settings (UseSudo etc.)

        if (Utils.isIpv4OrIpv6Address(hostAddressField.text)) {
            ConfigManager.setHostSettings(root.hostId, hostIdField.text, JSON.stringify({
                address: hostAddressField.text,
            }))
        }
        else {
            ConfigManager.setHostSettings(root.hostId, hostIdField.text, JSON.stringify({
                fqdn: hostAddressField.text,
            }))
        }
        ConfigManager.end_host_configuration()
        root._loading = true
        
        root.configurationChanged()
    }

    onRejected: {
        ConfigManager.cancel_host_configuration()
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
            Layout.fillWidth: true

            Label {
                text: "Name"
            }

            TextField {
                id: hostIdField
                width: parent.width
                placeholderText: "Unique name for host..."
                text: root.hostId === "new-host-id" ? "" : root.hostId
                validator: RegularExpressionValidator {
                    regularExpression: /[a-zA-Z\d\-\.]+/
                }
                onTextChanged: updateOkButton()
            }
        }

        Column {
            Layout.fillWidth: true

            Label {
                text: "IP Address or domain name"
            }

            TextField {
                id: hostAddressField
                width: parent.width
                placeholderText: ""
                text: root.hostSettings.address === undefined ? root.hostSettings.fqdn : root.hostSettings.address 
                validator: RegularExpressionValidator {
                    regularExpression: /[\.\:a-zA-Z\d\-]+/
                }
                onTextChanged: updateOkButton()
            }
        }

        // Just for extra spacing
        Item {
            Layout.fillWidth: true
            height: Theme.spacingNormal
        }

        BigText {
            text: "Configuration groups"

            Layout.alignment: Qt.AlignHCenter
        }

        ColumnLayout {
            spacing: 0

            Layout.fillHeight: true
            Layout.fillWidth: true

            TabBar {
                id: tabBar
                currentIndex: 0
                height: 40

                Layout.fillWidth: true

                TabButton {
                    text: `Selected (${root._selectedGroups.length})`
                }

                TabButton {
                    text: `Available (${root._availableGroups.length})`
                }
            }

            StackLayout {
                id: tabStackLayout
                currentIndex: tabBar.currentIndex

                Layout.fillWidth: true
                Layout.fillHeight: true

                RowLayout {
                    BorderRectangle {
                        borderColor: Theme.borderColor
                        backgroundColor: Theme.backgroundColor
                        border: 1

                        Layout.fillHeight: true
                        Layout.fillWidth: true

                        ListView {
                            id: selectedGroupsList
                            anchors.fill: parent
                            clip: true
                            // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                            boundsBehavior: Flickable.StopAtBounds


                            ScrollBar.vertical: ScrollBar {
                                active: true
                            }

                            model: root._selectedGroups
                            delegate: ItemDelegate {
                                width: selectedGroupsList.width
                                text: modelData
                                highlighted: ListView.isCurrentItem
                                onClicked: selectedGroupsList.currentIndex = index
                            }
                        }
                    }

                    // Add, remove and configure buttons.
                    ColumnLayout {
                        width: root.buttonSize
                        spacing: Theme.spacingNormal
                        layoutDirection: Qt.LeftToRight

                        Layout.fillHeight: true

                        ImageButton {
                            id: removeButton
                            enabled: selectedGroupsList.currentIndex !== -1
                            imageSource: "qrc:/main/images/button/remove"
                            size: root.buttonSize
                            onClicked: {
                                let selectedGroup = root._selectedGroups[selectedGroupsList.currentIndex]
                                ConfigManager.removeHostFromGroup(root.hostId, selectedGroup)
                                root.refreshGroups();
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
                    }
                }

                RowLayout {
                    BorderRectangle {
                        borderColor: Theme.borderColor
                        backgroundColor: Theme.backgroundColor
                        border: 1

                        Layout.fillHeight: true
                        Layout.fillWidth: true

                        ListView {
                            id: availableGroupsList
                            anchors.fill: parent
                            clip: true
                            // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                            boundsBehavior: Flickable.StopAtBounds

                            ScrollBar.vertical: ScrollBar {
                                active: true
                            }

                            model: root._availableGroups

                            delegate: ItemDelegate {
                                width: availableGroupsList.width
                                text: modelData
                                highlighted: ListView.isCurrentItem
                                onClicked: availableGroupsList.currentIndex = index
                            }
                        }
                    }

                    ColumnLayout {
                        width: root.buttonSize
                        height: tabStackLayout.height
                        spacing: Theme.spacingNormal

                        ImageButton {
                            id: addButton
                            enabled: availableGroupsList.currentIndex !== -1
                            imageSource: "qrc:/main/images/button/add"
                            size: root.buttonSize
                            onClicked: {
                                let selectedGroup = root._availableGroups[availableGroupsList.currentIndex]
                                ConfigManager.addHostToGroup(root.hostId, selectedGroup)
                                refreshGroups()
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

                            onClicked: groupAddDialog.open()
                        }

                        ImageButton {
                            id: deleteGroupButton
                            imageSource: "qrc:/main/images/button/delete"
                            tooltip: "Delete group"
                            size: root.buttonSize

                            onClicked: {
                                let selectedGroup = root._availableGroups[availableGroupsList.currentIndex]
                                ConfigManager.begin_group_configuration()
                                ConfigManager.remove_group(selectedGroup)
                                ConfigManager.endGroupConfiguration()
                                refreshGroups()
                            }
                        }
                    }
                }
            }
        }
    }

    GroupConfigurationDialog {
        id: groupConfigDialog
        visible: false
        // contentItem's margin seems to affect this dialog's position, so compensating for it here.
        x: -100 + Theme.marginDialog
        groupName: selectedGroupsList.currentIndex === -1 ? "" : root._selectedGroups[selectedGroupsList.currentIndex]
    }

    InputDialog {
        id: groupAddDialog
        visible: false
        width: parent.width
        height: 150

        inputSpecs: [{
            label: "Configuration group name",
            field_type: "Text",
            validator_regexp: "[a-zA-Z\d\-]+",
        }]

        onInputValuesGiven: function(inputValues) {
            ConfigManager.addGroup(inputValues[0])
            ConfigManager.endGroupConfiguration()
            refreshGroups()
        }

        onOpened: {
            ConfigManager.begin_group_configuration()
        }

        onRejected: {
            ConfigManager.cancel_group_configuration()
        }
    }

    // Forces re-evaluation of lists.
    function refreshGroups() {
        root._selectedGroups = ConfigManager.getSelectedGroups(root.hostId)
        root._availableGroups = ConfigManager.getAvailableGroups(root.hostId)
    }

    function fieldsAreValid() {
        return hostIdField.acceptableInput && hostAddressField.acceptableInput
    }

    function updateOkButton() {
        root.standardButton(Dialog.Ok).enabled = fieldsAreValid()
    }
}