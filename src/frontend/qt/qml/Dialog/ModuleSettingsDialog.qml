import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

import "../Text"


// This component should be a direct child of main window.
Dialog {
    id: root
    property string groupName: ""
    property string moduleId: ""
    property string moduleType: ""

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel
    background: DialogBackground { }

    contentItem: ScrollView {
        contentWidth: availableWidth

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 30

            Repeater {
                Layout.fillWidth: true

                model: getModuleSettings()

                Row {
                    Layout.fillWidth: true
                    height: column.implicitHeight
                    spacing: Theme.spacing_normal()

                    Column {
                        id: column
                        width: root.width * 0.5

                        Label {
                            width: parent.width
                            text: modelData.key
                        }

                        SmallText {
                            width: parent.width
                            text: modelData.description
                            color: Theme.color_dark_text()
                            wrapMode: Text.WordWrap
                        }
                    }

                    TextField {
                        id: textField
                        width: parent.width * 0.4
                        placeholderText: "(default)"
                        text: modelData.value

                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }

    function getModuleSettings() {
        let settings = ConfigManager.get_all_module_settings(root.moduleType, root.moduleId)
        let settingsArray = []
        for (let key in settings) {
            let value = ""
            if (root.moduleType === "connector") {
                value = ConfigManager.get_group_connector_setting(root.groupName, root.moduleId, key)
            }
            else if (root.moduleType === "monitor") {
                value = ConfigManager.get_group_monitor_setting(root.groupName, root.moduleId, key)
            }
            else if (root.moduleType === "command") {
                value = ConfigManager.get_group_command_setting(root.groupName, root.moduleId, key)
            }
            settingsArray.push({ "key": key, "description": settings[key], "value": value })
        }
        return settingsArray
    }
}