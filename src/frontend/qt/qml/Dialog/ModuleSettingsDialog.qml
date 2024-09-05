import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../StyleOverride"
import "../Text"
import "../Button"
import ".."


// This component should be a direct child of main window.
LightkeeperDialog {
    id: root
    property string groupName: ""
    property string moduleId: ""
    property string moduleType: ""
    property bool _loading: groupName === "" || moduleId === "" || moduleType === ""


    title: `Module settings: ${root.moduleId}`
    implicitWidth: 600
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

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
                LK.config.set_group_connector_setting(root.groupName, root.moduleId, key, value)
            }
            else if (root.moduleType === "monitor") {
                LK.config.set_group_monitor_setting(root.groupName, root.moduleId, key, value)
            }
            else if (root.moduleType === "command") {
                LK.config.set_group_command_setting(root.groupName, root.moduleId, key, value)
            }
        }

        root.configSaved(root.moduleType, root.groupName, root.moduleId)
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: scrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root._loading
    }

    contentItem: ScrollView {
        id: scrollView
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        contentWidth: availableWidth
        clip: true

        Column {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.marginScrollbar
            spacing: Theme.spacingNormal

            Repeater {
                id: repeater
                model: []

                RowLayout {
                    id: rowLayout
                    width: parent.width
                    height: textContainer.implicitHeight
                    spacing: Theme.spacingNormal

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
                            color: Theme.textColorDark
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
                        enabled: toggleSwitch.checked && !fileChooserButton.visible
                        placeholderText: toggleSwitch.checked ? "" : "unset"
                        placeholderTextColor: Theme.textColorDark
                        text: toggleSwitch.checked ? modelData.value : ""

                        Layout.preferredWidth: {
                            if (fileChooserButton.visible) {
                                scrollView.width * 0.35 - fileChooserButton.width - rowLayout.spacing
                            }
                            else {
                                scrollView.width * 0.35
                            }
                        }
                        Layout.alignment: Qt.AlignVCenter

                        Connections {
                            target: DesktopPortal
                            function onFileChooserResponse(token, filePath) {
                                if (fileChooserButton.visible && token === fileChooserButton._fileChooserToken) {
                                    textField.text = filePath
                                }
                            }
                        }
                    }

                    ImageButton {
                        id: fileChooserButton
                        property string _fileChooserToken: ""

                        // TODO: this is quick and hacky, refactor.
                        visible: modelData.key.endsWith("_path")
                        enabled: toggleSwitch.checked
                        imageSource: "qrc:/main/images/button/document-open-folder"
                        size: textField.implicitHeight * 0.8
                        onClicked: {
                            _fileChooserToken = DesktopPortal.openFileChooser()
                        }

                        Layout.preferredWidth: textField.implicitHeight
                        Layout.alignment: Qt.AlignVCenter
                    }

                }
            }
        }
    }

    // TODO: implement model in rust?
    function getModuleSettingsModel() {
        let settings = LK.config.get_all_module_settings(root.moduleType, root.moduleId)
        let settingsArray = []
        for (let key in settings) {
            let value = ""
            let enabled = true
            if (root.moduleType === "connector") {
                value = LK.config.get_group_connector_setting(root.groupName, root.moduleId, key)
            }
            else if (root.moduleType === "monitor") {
                value = LK.config.get_group_monitor_setting(root.groupName, root.moduleId, key)
            }
            else if (root.moduleType === "command") {
                value = LK.config.get_group_command_setting(root.groupName, root.moduleId, key)
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