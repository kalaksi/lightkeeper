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
    standardButtons: Dialog.Ok | Dialog.Cancel

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

                    RowHighlight {
                        width: parent.width
                        height: connectorRow.height + Theme.common_spacing() / 2

                        Row {
                            id: connectorRow
                            spacing: Theme.common_spacing()

                            NormalText {
                                text: modelData
                            }

                            SmallerText {
                                text: ConfigManager.get_group_connector_enabled(root.groupName, modelData) === "false" ? "disabled" : "enabled"
                                color: text === "disabled" ? Theme.color_red() : Theme.color_green()
                                anchors.verticalCenter: parent.verticalCenter
                            }
                        }
                    }

                    Repeater {
                        id: connectorSettingsRepeater
                        property string connectorName: modelData
                        model: ConfigManager.get_group_connector_settings_keys(root.groupName, modelData)

                        Layout.fillWidth: true
                        RowLayout {
                            NormalText {
                                Layout.fillWidth: true

                                Layout.leftMargin: Theme.common_indentation()
                                text: modelData + ": "
                            }

                            NormalText {
                                Layout.fillWidth: true
                                text: ConfigManager.get_group_connector_setting(root.groupName, connectorSettingsRepeater.connectorName, modelData)
                            }
                        }
                    }
                }
            }

            BigText {
                topPadding: Theme.form_row_spacing()
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

                    RowHighlight {
                        id: monitorHighlighter
                        width: parent.width
                        height: monitoringModuleRow.height + Theme.common_spacing() / 2

                        RowLayout {
                            id: monitoringModuleRow
                            width: parent.width
                            spacing: Theme.common_spacing()

                            NormalText {
                                text: modelData

                                Layout.alignment: Qt.AlignVCenter
                            }

                            SmallerText {
                                text: ConfigManager.get_group_monitor_enabled(root.groupName, modelData) === "false" ? "disabled" : "enabled"
                                color: text === "disabled" ? Theme.color_red() : Theme.color_green()

                                Layout.fillWidth: true
                                Layout.alignment: Qt.AlignVCenter
                            }

                            ImageButton {
                                visible: monitorHighlighter.containsMouse
                                imageSource: "qrc:/main/images/button/story-editor"
                                onClicked: groupConfigDialog.open()
                                flatButton: false
                                roundButton: true
                                tooltip: "Module settings..."
                                hoverEnabled: true

                                Layout.fillHeight: true
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
                            NormalText {
                                Layout.fillWidth: true

                                Layout.leftMargin: Theme.common_indentation()
                                text: modelData + ": "
                            }

                            NormalText {
                                Layout.fillWidth: true
                                text: ConfigManager.get_group_monitor_setting(root.groupName, monitorSettingsRepeater.monitorName, modelData)
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
}