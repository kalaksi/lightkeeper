import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Button"
import "../Text"
import ".."

// This component should be a direct child of main window.
Dialog {
    id: root
    required property string groupName 
    property var _connectorList: []
    property var _monitorList: []
    property var _commandList: []
    property bool _loading: true

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    background: DialogBackground { }
    standardButtons: Dialog.Cancel | Dialog.Ok

    WorkingSprite {
        visible: root._loading
    }

    contentItem: ScrollView {
        contentWidth: availableWidth

        Column {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            spacing: Theme.spacing_tight()

            BigText {
                text: "Connector module settings"
            }

            OptionalText {
                visible: root._connectorList.length === 0
                anchors.left: parent.left
                anchors.leftMargin: Theme.common_indentation()

                placeholder: "No changes"
                text: ""
            }

            Repeater {
                id: connectorRepeater
                model: root._connectorList

                Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.common_indentation()

                    RowHighlight {
                        id: connectorHighlighter
                        width: parent.width
                        height: connectoringModuleRow.height

                        RowLayout {
                            id: connectoringModuleRow
                            width: parent.width
                            spacing: Theme.spacing_tight()

                            NormalText {
                                text: modelData
                                // Has to be set explicitly or may, for some reason, change color when redrawn.
                                color: Theme.color_text()

                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignVCenter
                            }

                            ImageButton {
                                imageSource: "qrc:/main/images/button/entry-edit"
                                onClicked: {
                                    moduleSettingsDialog.moduleId = modelData
                                    moduleSettingsDialog.moduleType = "connector"
                                    moduleSettingsDialog.visible = true
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Module settings..."
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }

                            ImageButton {
                                imageSource: "qrc:/main/images/button/delete"
                                onClicked: {
                                    ConfigManager.remove_group_connector(root.groupName, modelData)
                                    root._connectorList = ConfigManager.get_group_connectors(root.groupName)
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Remove module from group"
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }
                        }
                    }

                    Repeater {
                        id: connectorSettingsRepeater
                        property string connectorName: modelData
                        model: ConfigManager.get_group_connector_settings_keys(root.groupName, connectorName)

                        RowLayout {
                            SmallText {
                                text: modelData + ": "
                                color: Theme.color_dark_text()

                                Layout.leftMargin: Theme.common_indentation()
                            }

                            SmallText {
                                text: ConfigManager.get_group_connector_setting(root.groupName, connectorSettingsRepeater.connectorName, modelData)
                                color: Theme.color_dark_text()

                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }

            RowLayout {
                width: parent.width

                BigText {
                    topPadding: Theme.spacing_loose()
                    text: "Enabled monitoring modules and settings"

                    Layout.fillWidth: true
                }

                ImageButton {
                    imageSource: "qrc:/main/images/button/add"
                    onClicked: {
                        let allMonitors = ConfigManager.get_all_monitors()
                        moduleAddDialog.inputSpecs = []
                        moduleAddDialog.inputSpecs = [{
                            label: "Monitoring module",
                            field_type: "Option",
                            options: allMonitors,
                            option_descriptions: allMonitors.map((monitor) => ConfigManager.get_monitor_description(monitor))
                        }]
                        moduleAddDialog.onInputValuesGiven.connect((inputValues) => {
                            ConfigManager.add_group_monitor(root.groupName, inputValues[0])
                            refreshMonitorList()
                        })
                        moduleAddDialog.open()
                    }
                    flatButton: true
                    roundButton: false
                    tooltip: "Add new module"
                    width: 26

                    Layout.alignment: Qt.AlignBottom
                }
            }

            OptionalText {
                visible: monitorRepeater.model.length === 0
                anchors.left: parent.left
                anchors.leftMargin: Theme.common_indentation()

                placeholder: "No changes"
                text: ""
            }

            Repeater {
                id: monitorRepeater
                model: root._monitorList

                Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.common_indentation()

                    RowHighlight {
                        id: monitorHighlighter
                        width: parent.width
                        height: monitoringModuleRow.height

                        RowLayout {
                            id: monitoringModuleRow
                            width: parent.width
                            spacing: Theme.spacing_tight()

                            NormalText {
                                text: modelData

                                Layout.alignment: Qt.AlignVCenter
                                Layout.rightMargin: Theme.spacing_normal()
                            }

                            /* 
                            See comment below
                            PixelatedText {
                                id: monitorStatusText
                                text: ConfigManager.get_group_monitor_enabled(root.groupName, modelData) === "true" ? "Enabled" : "Disabled"
                                color: text === "Enabled" ? Theme.color_green() : Theme.color_red()
                            }
                            */

                            // Spacer
                            Item {
                                Layout.fillWidth: true
                            }

                            /*
                            Control if module will be enabled or disabled (previous enable overridden).
                            Could be useful but currently it might just confuse the user more than help,
                            since the module settings have a similar switch that works a bit differently.

                            Switch {
                                checked: ConfigManager.get_group_monitor_enabled(root.groupName, modelData) === "true"
                                onClicked: {
                                    ConfigManager.toggle_group_monitor_enabled(root.groupName, modelData)
                                    refreshMonitorList()
                                }

                                Layout.alignment: Qt.AlignVCenter
                                Layout.rightMargin: Theme.spacing_loose()
                            }
                            */

                            ImageButton {
                                imageSource: "qrc:/main/images/button/entry-edit"
                                onClicked: {
                                    moduleSettingsDialog.moduleId = modelData
                                    moduleSettingsDialog.moduleType = "monitor"
                                    moduleSettingsDialog.visible = true
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Module settings..."
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }

                            ImageButton {
                                imageSource: "qrc:/main/images/button/delete"
                                onClicked: {
                                    ConfigManager.remove_group_monitor(root.groupName, modelData)
                                    refreshMonitorList()
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Remove module from group"
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }
                        }
                    }

                    Repeater {
                        id: monitorSettingsRepeater
                        property string monitorName: modelData
                        model: ConfigManager.get_group_monitor_settings_keys(root.groupName, monitorName)

                        RowLayout {
                            SmallText {
                                text: modelData + ": "
                                color: Theme.color_dark_text()

                                Layout.fillWidth: true
                                Layout.leftMargin: Theme.common_indentation()
                            }

                            SmallText {
                                text: ConfigManager.get_group_monitor_setting(root.groupName, monitorSettingsRepeater.monitorName, modelData)
                                color: Theme.color_dark_text()

                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }

            RowLayout {
                width: parent.width

                BigText {
                    topPadding: Theme.spacing_loose()
                    text: "Enabled command modules and settings"

                    Layout.fillWidth: true
                }

                ImageButton {
                    imageSource: "qrc:/main/images/button/add"
                    onClicked: {
                        let allCommands = ConfigManager.get_all_commands()
                        moduleAddDialog.inputSpecs = []
                        moduleAddDialog.inputSpecs = [{
                            label: "Command module",
                            field_type: "Option",
                            options: allCommands,
                            option_descriptions: allCommands.map((command) => ConfigManager.get_command_description(command))
                        }]
                        moduleAddDialog.onInputValuesGiven.connect((inputValues) => {
                            ConfigManager.add_group_command(root.groupName, inputValues[0])
                            refreshCommandList()
                        })
                        moduleAddDialog.open()
                    }
                    flatButton: true
                    roundButton: false
                    tooltip: "Add new module"
                    width: 26

                    Layout.alignment: Qt.AlignBottom
                }
            }

            OptionalText {
                visible: commandRepeater.model.length === 0
                anchors.left: parent.left
                anchors.leftMargin: Theme.common_indentation()

                placeholder: "No changes"
                text: ""
            }

            Repeater {
                id: commandRepeater
                model: root._commandList

                Column {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.leftMargin: Theme.common_indentation()

                    RowHighlight {
                        id: commandHighlighter
                        width: parent.width
                        height: commandModuleRow.height

                        RowLayout {
                            id: commandModuleRow
                            width: parent.width
                            spacing: Theme.spacing_tight()

                            NormalText {
                                text: modelData

                                Layout.alignment: Qt.AlignVCenter
                                Layout.rightMargin: Theme.spacing_normal()
                            }

                            // Spacer
                            Item {
                                Layout.fillWidth: true
                            }

                            ImageButton {
                                imageSource: "qrc:/main/images/button/entry-edit"
                                onClicked: {
                                    moduleSettingsDialog.moduleId = modelData
                                    moduleSettingsDialog.moduleType = "command"
                                    moduleSettingsDialog.visible = true
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Module settings..."
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }

                            ImageButton {
                                imageSource: "qrc:/main/images/button/delete"
                                onClicked: {
                                    ConfigManager.remove_group_command(root.groupName, modelData)
                                    root._commandList = ConfigManager.get_group_commands(root.groupName)
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Remove module from group"
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }
                        }
                    }

                    Repeater {
                        id: commandSettingsRepeater
                        property string commandName: modelData
                        model: ConfigManager.get_group_command_settings_keys(root.groupName, commandName)

                        RowLayout {
                            SmallText {
                                text: modelData + ": "
                                color: Theme.color_dark_text()

                                Layout.fillWidth: true
                                Layout.leftMargin: Theme.common_indentation()
                            }

                            SmallText {
                                text: ConfigManager.get_group_command_setting(root.groupName, commandSettingsRepeater.commandName, modelData)
                                color: Theme.color_dark_text()

                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }
        }
    }

    onOpened: {
        ConfigManager.begin_group_configuration()
        root._connectorList = ConfigManager.get_group_connectors(root.groupName) 
        root._monitorList = ConfigManager.get_group_monitors(root.groupName)
        root._commandList = ConfigManager.get_group_commands(root.groupName)
        root._loading = false
    }

    onAccepted: {
        ConfigManager.commit_group_configuration()
        root._loading = true
    }

    onRejected: {
        ConfigManager.cancel_group_configuration()
        root._loading = true
    }

    ModuleSettingsDialog {
        id: moduleSettingsDialog
        visible: false
        groupName: root.groupName

        onConfigSaved: function(moduleType, groupName, moduleId) {
            if (moduleType === "connector") {
                root._connectorList = []
                root._connectorList = ConfigManager.get_group_connectors(groupName)
            } else if (moduleType === "monitor") {
                refreshMonitorList()
            }
        }
    }

    InputDialog {
        id: moduleAddDialog
        visible: false
        width: parent.width
        height: 200
    }

    function refreshMonitorList() {
        root._monitorList = []
        root._monitorList = ConfigManager.get_group_monitors(root.groupName)
    }

    function refreshCommandList() {
        root._commandList = []
        root._commandList = ConfigManager.get_group_commands(root.groupName)
    }
}