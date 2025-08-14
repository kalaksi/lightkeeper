import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Text"
import "../Button"
import ".."
import "../StyleOverride"


LightkeeperDialog {
    id: root
    property string moduleId: ""
    property alias moduleSettings: repeater.model
    property bool _loading: moduleId === ""

    title: `Module settings: ${root.moduleId}`
    implicitWidth: 600
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal settingsUpdated(string moduleId, var settings)

    onAccepted: {
        let moduleSettings = []
        for (let i = 0; i < repeater.model.length; i++) {
            let nextItem = repeater.itemAt(i)
            // See `ModuleSetting` in ConfigManagerModel for the model.
            let moduleSetting = {
                "key": nextItem.children[0].children[0].text,
                "value": nextItem.children[2].text,
                "enabled": nextItem.children[1].checked,
                // Not used.
                "description": "",
            }
            moduleSettings.push(moduleSetting)
        }

        root.settingsUpdated(root.moduleId, moduleSettings)
        root.resetModel()
    }

    onRejected: {
        root.resetModel()
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
                        echoMode: modelData.key.endsWith("_password") || modelData.key.endsWith("_passphrase") ?
                            TextInput.Password : TextInput.Normal

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

    function resetModel() {
        root.moduleSettings = []
    }
}