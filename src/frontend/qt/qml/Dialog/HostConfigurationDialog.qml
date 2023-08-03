import QtQuick 2.15
import QtQuick.Controls 1.4
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Button"

// This component should be a direct child of main window.
Dialog {
    id: root
    required property string hostName
    property var hostSettings: JSON.parse(ConfigManager.get_host_settings(hostName))
    property var _selectedGroups: ConfigManager.get_selected_groups(hostName)
    property var _availableGroups: ConfigManager.get_available_groups(hostName)
    property int _contentWidth: 360
    property int buttonSize: 42

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    bottomMargin: 80
    background: DialogBackground { }
    standardButtons: Dialog.Ok | Dialog.Cancel

    contentItem: ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.dialog_margin()
        spacing: Theme.form_row_spacing()

        Column {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth

            Label {
                text: "Name"
            }

            TextField {
                width: parent.width
                placeholderText: "Unique name for host..."
                text: root.hostName
                // TODO: validation
            }
        }

        Column {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth

            Label {
                text: "IP Address or domain name"
            }

            TextField {
                width: parent.width
                placeholderText: ""
                text: root.hostSettings.address === "0.0.0.0" ? root.hostSettings.fqdn : root.hostSettings.address 
            }
        }

        // Just for extra spacing
        Item {
            Layout.fillWidth: true
            height: Theme.form_row_spacing()
        }

        Row {
            spacing: Theme.common_spacing()

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
                    title: "Selected groups"

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
                    title: "Available groups"

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
            Column {
                width: configButton.width
                height: tabView.height
                spacing: Theme.common_spacing()
                topPadding: 30

                property bool isValidGroupSelection: tabView._selectedGroup !== "" && tabView._selectedGroupTab === tabView.currentIndex

                ImageButton {
                    id: addButton
                    visible: tabView.currentIndex === 1
                    enabled: parent.isValidGroupSelection
                    imageSource: "qrc:/main/images/button/add"
                    width: root.buttonSize
                    onClicked: {
                        ConfigManager.add_host_to_group(root.hostName, tabView._selectedGroup)
                        // Forces re-evaluation of lists.
                        root._selectedGroups = ConfigManager.get_selected_groups(root.hostName)
                        root._availableGroups = ConfigManager.get_available_groups(root.hostName)
                    }
                }

                ImageButton {
                    id: removeButton
                    visible: tabView.currentIndex === 0
                    enabled: parent.isValidGroupSelection
                    imageSource: "qrc:/main/images/button/remove"
                    width: root.buttonSize
                    onClicked: {
                        ConfigManager.remove_host_from_group(root.hostName, tabView._selectedGroup)
                        // Forces re-evaluation of lists.
                        root._selectedGroups = ConfigManager.get_selected_groups(root.hostName)
                        root._availableGroups = ConfigManager.get_available_groups(root.hostName)
                    }
                }

                ImageButton {
                    id: configButton
                    enabled: parent.isValidGroupSelection
                    imageSource: "qrc:/main/images/button/configure"
                    width: root.buttonSize
                    onClicked: groupConfigDialog.open()
                }
            }
        }

        // Content will overflow behind the buttons with Layout.fillHeight (ugh...), reserve some space with them with this.
        Item {
            Layout.fillWidth: true
            height: 40
        }
    }

    GroupConfigurationDialog {
        id: groupConfigDialog
        visible: false
        groupName: tabView._selectedGroup
    }
}