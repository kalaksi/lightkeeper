import QtQuick 2.15
import QtQuick.Controls 2.15
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

    signal configSaved(string moduleType, string groupName, string moduleId)

    contentItem: ScrollView {
        contentWidth: availableWidth

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 30

            Repeater {
                id: repeater
                Layout.fillWidth: true

                model: getModuleSettingsModel()

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

                    CheckBox {
                        id: checkBox
                        anchors.verticalCenter: parent.verticalCenter
                        checkState: modelData.enabled === false ? Qt.Unchecked : Qt.Checked
                    }

                    TextField {
                        id: textField
                        width: parent.width * 0.4
                        anchors.verticalCenter: parent.verticalCenter
                        enabled: checkBox.checkState === Qt.Checked
                        placeholderText: checkBox.checkState === Qt.Checked ? "" : "unset"
                        text: checkBox.checkState === Qt.Checked ? modelData.value : ""

                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }

    onAccepted: {
        for (let i = 0; i < repeater.model.length; i++) {
            let nextItem = repeater.itemAt(i)
            let key = nextItem.children[0].children[0].text
            let enabled = nextItem.children[1].checkState === Qt.Checked
            let value = nextItem.children[2].text
            let previousEnabled = repeater.model.filter((item) => item.key === key)[0].enabled
            let previousValue = repeater.model.filter((item) => item.key === key)[0].value

            if (value === previousValue && enabled === previousEnabled) {
                continue
            }

            // "unset" is currently used as a special value to act as a null value.
            if (enabled === false) {
                value = "unset"
            }

            if (root.moduleType === "connector") {
                ConfigManager.set_group_connector_setting(root.groupName, root.moduleId, key, value)
            }
            else if (root.moduleType === "monitor") {
                ConfigManager.set_group_monitor_setting(root.groupName, root.moduleId, key, value)
            }
            else if (root.moduleType === "command") {
                ConfigManager.set_group_command_setting(root.groupName, root.moduleId, key, value)
            }
        }

        root.configSaved(root.moduleType, root.groupName, root.moduleId)
        repeater.model = getModuleSettingsModel()
    }

    function getModuleSettingsModel() {
        let settings = ConfigManager.get_all_module_settings(root.moduleType, root.moduleId)
        let settingsArray = []
        for (let key in settings) {
            let value = ""
            let enabled = true
            if (root.moduleType === "connector") {
                value = ConfigManager.get_group_connector_setting(root.groupName, root.moduleId, key)
            }
            else if (root.moduleType === "monitor") {
                value = ConfigManager.get_group_monitor_setting(root.groupName, root.moduleId, key)
            }
            else if (root.moduleType === "command") {
                value = ConfigManager.get_group_command_setting(root.groupName, root.moduleId, key)
            }

            if (value === "unset") {
                value = ""
                enabled = false
            }
            
            settingsArray.push({
                "key": key,
                "description": settings[key],
                "value": value,
                "enabled": enabled
            })
        }
        return settingsArray
    }
}