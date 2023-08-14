import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Text"
import ".."


// This component should be a direct child of main window.
Dialog {
    id: root
    property string groupName: ""
    property string moduleId: ""
    property string moduleType: ""
    property bool _loading: groupName === "" || moduleId === "" || moduleType === ""

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel
    background: DialogBackground { }

    signal configSaved(string moduleType, string groupName, string moduleId)

    onOpened: {
        repeater.model = []
        repeater.model = getModuleSettingsModel()
    }

    onAccepted: {
        for (let i = 0; i < repeater.model.length; i++) {
            let nextItem = repeater.itemAt(i)
            let key = nextItem.children[0].children[0].text
            let enabled = nextItem.children[1].checked
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
    }


    WorkingSprite {
        visible: root._loading
    }

    contentItem: ScrollView {
        contentWidth: availableWidth

        BigText {
            text: `Module settings: ${root.moduleId}`

            Layout.alignment: Qt.AlignHCenter
            Layout.bottomMargin: Theme.spacing_loose()
        }

        Column {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.margins: Theme.margin_dialog()
            spacing: Theme.spacing_normal()

            Repeater {
                id: repeater
                model: []

                RowLayout {
                    width: parent.width
                    height: textContainer.implicitHeight
                    spacing: Theme.spacing_normal()

                    Column {
                        id: textContainer
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter

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

                    Switch {
                        id: toggleSwitch
                        checked: modelData.enabled

                        Layout.alignment: Qt.AlignVCenter
                    }

                    TextField {
                        id: textField
                        enabled: toggleSwitch.checked
                        placeholderText: toggleSwitch.checked ? "" : "unset"
                        text: toggleSwitch.checked ? modelData.value : ""

                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                    }
                }
            }
        }
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