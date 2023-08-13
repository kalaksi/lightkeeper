import QtQuick 2.15
import QtQuick.Controls 1.4
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Button"
import "../Text"
import "../js/Utils.js" as Utils
import ".."

// This component should be a direct child of main window.
Dialog {
    id: root
    property string hostId: ""
    property int buttonSize: 42
    property var hostSettings: JSON.parse(ConfigManager.get_host_settings(hostId))
    property var _selectedGroups: ConfigManager.get_selected_groups(hostId)
    property var _availableGroups: ConfigManager.get_available_groups(hostId)
    property int _contentWidth: 360
    property bool _loading: true

    modal: true
    implicitWidth: 550
    implicitHeight: 650
    background: DialogBackground { }
    standardButtons: Dialog.Ok | Dialog.Cancel

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
        anchors.margins: Theme.margin_dialog()
        anchors.bottomMargin: Theme.margin_dialog_bottom()
        spacing: Theme.spacing_loose()

        BigText {
            text: "Host details"

            Layout.alignment: Qt.AlignHCenter
        }

        Column {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth

            Label {
                text: "Name"
            }

            TextField {
                id: hostIdField
                width: parent.width
                placeholderText: "Unique name for host..."
                text: root.hostId === "new-host-id" ? "" : root.hostId
                validator: RegularExpressionValidator {
                    regularExpression: /[a-zA-Z\d\-]+/
                }
            }
        }

        Column {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth

            Label {
                text: "IP Address or domain name"
            }

            TextField {
                id: hostAddressField
                width: parent.width
                placeholderText: ""
                text: root.hostSettings.address === undefined ? root.hostSettings.fqdn : root.hostSettings.address 
                validator: RegularExpressionValidator {
                    regularExpression: /[\.\:a-zA-Z\d]+/
                }
            }
        }

        // Just for extra spacing
        Item {
            Layout.fillWidth: true
            height: Theme.spacing_normal()
        }

        BigText {
            text: "Configuration groups"

            Layout.alignment: Qt.AlignHCenter
        }

        Row {
            spacing: Theme.spacing_normal()

            Layout.alignment: Qt.AlignHCenter
            Layout.fillHeight: true
            Layout.preferredWidth: root._contentWidth

            TabView {
                id: tabView
                width: parent.width
                height: parent.height

                property string _selectedGroup: ""
                // Clearing _selectedGroup on tab change would be simpler, but couldn't find a way to detect a tab change.
                property int _selectedGroupTab: -1


                Tab {
                    title: `Selected (${root._selectedGroups.length})    `

                    ListView {
                        id: selectedGroupsList
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
                            highlighted: tabView.isSelected(modelData)
                            onClicked: tabView.selectGroup(modelData, tabView.currentIndex)
                        }
                    }
                }

                Tab {
                    title: `Available (${root._availableGroups.length})`

                    ListView {
                        id: availableGroupsList
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
                            highlighted: tabView.isSelected(modelData)
                            onClicked: tabView.selectGroup(modelData, tabView.currentIndex)
                        }
                    }
                }

                function selectGroup(group, tabIndex) {
                    if (tabView.isSelected(group)) {
                        tabView._selectedGroup = ""
                        tabView._selectedGroupTab = -1
                    }
                    else {
                        tabView._selectedGroup = group 
                        tabView._selectedGroupTab = tabIndex
                    }
                }

                function isSelected(group) {
                    return tabView._selectedGroup === group &&
                           tabView._selectedGroupTab === tabView.currentIndex
                }
            }

            // Add, remove and configure buttons.
            ColumnLayout {
                width: configButton.width
                height: tabView.height
                spacing: Theme.spacing_normal()

                property bool isValidGroupSelection: tabView._selectedGroup !== "" && tabView._selectedGroupTab === tabView.currentIndex

                ImageButton {
                    id: addButton
                    visible: tabView.currentIndex === 1
                    enabled: parent.isValidGroupSelection
                    imageSource: "qrc:/main/images/button/add"
                    width: root.buttonSize
                    onClicked: {
                        ConfigManager.add_host_to_group(root.hostId, tabView._selectedGroup)
                        // Forces re-evaluation of lists.
                        root._selectedGroups = ConfigManager.get_selected_groups(root.hostId)
                        root._availableGroups = ConfigManager.get_available_groups(root.hostId)
                    }

                    Layout.topMargin: 30
                }

                ImageButton {
                    id: removeButton
                    visible: tabView.currentIndex === 0
                    enabled: parent.isValidGroupSelection
                    imageSource: "qrc:/main/images/button/remove"
                    width: root.buttonSize
                    onClicked: {
                        ConfigManager.remove_host_from_group(root.hostId, tabView._selectedGroup)
                        // Forces re-evaluation of lists.
                        root._selectedGroups = ConfigManager.get_selected_groups(root.hostId)
                        root._availableGroups = ConfigManager.get_available_groups(root.hostId)
                    }

                    Layout.topMargin: 30
                }

                ImageButton {
                    id: configButton
                    enabled: parent.isValidGroupSelection
                    imageSource: "qrc:/main/images/button/configure"
                    width: root.buttonSize
                    onClicked: groupConfigDialog.open()
                }

                // Spacer
                Item {
                    Layout.fillHeight: true
                }

                ImageButton {
                    id: createGroupButton
                    visible: tabView.currentIndex === 1
                    imageSource: "qrc:/main/images/button/add"
                    width: root.buttonSize
                }

                ImageButton {
                    id: deleteGroupButton
                    visible: tabView.currentIndex === 1
                    imageSource: "qrc:/main/images/button/remove"
                    width: root.buttonSize
                }
            }
        }
    }

    onOpened: {
        ConfigManager.begin_host_configuration()
        if (root.hostId === "") {
            ConfigManager.add_host("new-host-id")
            root.hostId = "new-host-id"
        }
        root._loading = false
    }

    onAccepted: {
        // TODO: GUI for host settings (UseSudo etc.)

        if (Utils.isIpv4OrIpv6Address(hostAddressField.text)) {
            ConfigManager.set_host_settings(root.hostId, hostIdField.text, JSON.stringify({
                address: hostAddressField.text,
            }))
        }
        else {
            ConfigManager.set_host_settings(root.hostId, hostIdField.text, JSON.stringify({
                fqdn: hostAddressField.text,
            }))
        }
        ConfigManager.end_host_configuration()
        root._loading = true
    }

    onRejected: {
        ConfigManager.cancel_host_configuration()
        root._loading = true
    }

    GroupConfigurationDialog {
        id: groupConfigDialog
        visible: false
        groupName: tabView._selectedGroup
    }
}