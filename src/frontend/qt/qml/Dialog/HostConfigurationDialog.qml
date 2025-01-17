import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

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
        refreshGroups()
        updateOkButton()
        root._loading = false
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
                onTextChanged: updateOkButton()
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
                onTextChanged: updateOkButton()
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
                onTextChanged: updateOkButton()
            }
        }

        Row {
            Layout.fillWidth: true

            Switch {
                id: useSudoCheckbox
                enabled: false
                checked: true
                onCheckedChanged: updateOkButton()
            }

            Label {
                anchors.verticalCenter: parent.verticalCenter
                text: "Use sudo"
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

                    Rectangle {
                        color: Theme.backgroundColor
                        border.color: Theme.borderColor
                        border.width: 1

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
                                LK.config.removeHostFromGroup(root.hostId, selectedGroup)
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
                    Rectangle {
                        color: Theme.backgroundColor
                        border.color: Theme.borderColor
                        border.width: 1

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
                                LK.config.addHostToGroup(root.hostId, selectedGroup)
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
                                LK.config.beginGroupConfiguration()
                                LK.config.remove_group(selectedGroup)
                                LK.config.endGroupConfiguration()
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
        // contentItem's margin seems to affect this dialog's position, so compensating for it here.
        x: -100 + Theme.marginDialog
        anchors.centerIn: undefined
        groupName: selectedGroupsList.currentIndex === -1 ? "" : root._selectedGroups[selectedGroupsList.currentIndex]
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
            LK.config.endGroupConfiguration()
            refreshGroups()
        }

        onOpened: {
            LK.config.beginGroupConfiguration()
        }

        onRejected: {
            LK.config.cancelGroupConfiguration()
        }
    }

    // Forces re-evaluation of lists.
    function refreshGroups() {
        root._selectedGroups = LK.config.getSelectedGroups(root.hostId)
        root._availableGroups = LK.config.getAvailableGroups(root.hostId)
    }

    function fieldsAreValid() {
        return hostIdField.acceptableInput &&
               hostAddressField.acceptableInput
    }

    function updateOkButton() {
        root.standardButton(Dialog.Ok).enabled = fieldsAreValid()
    }
}