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
    property var _connectorList: ConfigManager.get_group_connectors(root.groupName) 
    property var _monitorList: ConfigManager.get_group_monitors(root.groupName)

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    background: DialogBackground { }
    standardButtons: Dialog.Cancel | Dialog.Ok


    contentItem: ScrollView {
        id: rootScrollView
        contentWidth: availableWidth

        ColumnLayout {
            anchors.fill: parent

            BigText {
                text: "Connector modules"
            }

            OptionalText {
                visible: root._connectorList.length === 0
                Layout.leftMargin: Theme.common_indentation()

                placeholder: "No changes"
                text: ""
            }

            Repeater {
                id: connectorRepeater
                model: root._connectorList

                Column {
                    Layout.leftMargin: Theme.common_indentation()
                    width: parent.width - 40
                    spacing: 0

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
                                Layout.rightMargin: Theme.common_indentation()
                            }
                        }
                    }

                    Repeater {
                        id: connectorSettingsRepeater
                        property string connectorName: modelData
                        model: ConfigManager.get_group_connector_settings_keys(root.groupName, connectorName)

                        Layout.fillWidth: true
                        RowLayout {
                            SmallText {
                                text: modelData + ": "
                                color: Theme.color_dark_text()

                                Layout.fillWidth: true
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

            BigText {
                topPadding: Theme.spacing_loose()
                text: "Monitoring modules"
            }

            OptionalText {
                visible: monitorRepeater.model.length === 0
                Layout.leftMargin: Theme.common_indentation()

                placeholder: "No changes"
                text: ""
            }

            Repeater {
                id: monitorRepeater
                model: root._monitorList

                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Column {
                    Layout.leftMargin: Theme.common_indentation()
                    width: parent.width - 40
                    spacing: 0

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
                                // Has to be set explicitly or may, for some reason, change color when redrawn.
                                color: Theme.color_text()

                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignVCenter
                            }

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
                                imageSource: "qrc:/main/images/button/cancel"
                                onClicked: {
                                    ConfigManager.toggle_group_monitor_enabled(root.groupName, modelData)
                                    root._monitorList = ConfigManager.get_group_monitors(root.groupName)
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Enable/disable module"
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                            }

                            ImageButton {
                                imageSource: "qrc:/main/images/button/delete"
                                onClicked: {
                                    ConfigManager.remove_group_monitor(root.groupName, modelData)
                                    root._monitorList = ConfigManager.get_group_monitors(root.groupName)
                                }
                                flatButton: true
                                roundButton: false
                                tooltip: "Remove module from group"
                                width: 26

                                Layout.alignment: Qt.AlignVCenter
                                Layout.rightMargin: Theme.common_indentation()
                            }
                        }
                    }

                    Repeater {
                        id: monitorSettingsRepeater
                        property string monitorName: modelData
                        model: ConfigManager.get_group_monitor_settings_keys(root.groupName, monitorName)

                        Layout.fillWidth: true
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

            // Content will overflow behind the buttons with Layout.fillHeight (ugh...), reserve some space with them with this.
            Item {
                Layout.fillWidth: true
                height: 40
            }
        }
    }

    ModuleSettingsDialog {
        id: moduleSettingsDialog
        visible: false
        groupName: root.groupName

        onAcceptedValid: function(moduleType, groupName, moduleId) {
            if (moduleType === "connector") {
                root._connectorList = []
                root._connectorList = ConfigManager.get_group_connectors(groupName)
            } else if (moduleType === "monitor") {
                root._monitorList = []
                root._monitorList = ConfigManager.get_group_monitors(groupName)
            }
        }
    }
}