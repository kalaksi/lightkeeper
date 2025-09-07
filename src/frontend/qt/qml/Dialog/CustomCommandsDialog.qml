import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Button"
import "../Misc"
import ".."

LightkeeperDialog {
    id: root
    property string hostId: ""
    property int buttonSize: 38
    property bool _loading: hostId === ""
    property alias customCommands: commandList.model

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    title: `Custom commands`
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged(string hostId)

    onOpened: {
        root.refreshModel()
        LK.config.beginHostConfiguration()
    }

    onAccepted: {
        let customCommandsJson = root.customCommands.map(JSON.stringify)
        LK.config.updateCustomCommands(root.hostId, customCommandsJson)
        LK.config.endHostConfiguration()
        root.configurationChanged(root.hostId)
        root.resetModel()
    }

    onRejected: {
        LK.config.cancelHostConfiguration()
        root.resetModel()
    }

    WorkingSprite {
        visible: root._loading
    }

    contentItem: Column {
        id: rootColumn
        visible: !root._loading
        spacing: Theme.spacingNormal
        anchors.right: parent.right
        anchors.left: parent.left
        anchors.margins: Theme.marginDialog

        RowLayout {
            width: parent.width
            height: parent.height * 0.95

            LKListView {
                id: commandList
                model: []
                labelPropertyName: "name"
                descriptionPropertyName: "description"
                property var selectedCommand: model[currentIndex]

                Layout.fillHeight: true
                Layout.fillWidth: true
            }

            // Add, remove and configure buttons.
            ColumnLayout {
                width: root.buttonSize
                spacing: Theme.spacingNormal
                layoutDirection: Qt.LeftToRight

                Layout.fillHeight: true

                ImageButton {
                    imageSource: "qrc:/main/images/button/add"
                    size: root.buttonSize
                    onClicked: {
                        commandAddDialog.inputSpecs = [
                            { label: "Command name", field_type: "Text" },
                            { label: "Description", field_type: "Text" },
                            { label: "Shell command", field_type: "Text" }
                        ]
                        commandAddDialog.open()
                    }
                }

                ImageButton {

                    imageSource: "qrc:/main/images/button/entry-edit"
                    size: root.buttonSize
                    onClicked: {
                        commandEditDialog.inputSpecs = [
                            { label: "Command name", field_type: "ReadOnlyText", default_value: commandList.selectedCommand.name },
                            { label: "Description", field_type: "Text", default_value: commandList.selectedCommand.description },
                            { label: "Shell command", field_type: "Text", default_value: commandList.selectedCommand.command }
                        ]
                        commandEditDialog.open()
                    }
                }

                ImageButton {
                    id: removeButton
                    imageSource: "qrc:/main/images/button/delete"
                    size: root.buttonSize
                    onClicked: {
                        root.removeCustomCommand(commandList.selectedCommand.name)
                    }  
                }

                // Spacer
                Item {
                    Layout.fillHeight: true
                }
            }
        }
    }

    InputDialog {
        id: commandAddDialog
        title: "Add command"
        width: 500
        height: 200
        onInputValuesGiven: function(inputValues) {
            root.addCustomCommand(inputValues[0], inputValues[1], inputValues[2])
        }
    }

    InputDialog {
        id: commandEditDialog
        title: "Edit command"
        width: 500
        height: 200
        onInputValuesGiven: function(inputValues) {
            root.editCustomCommand(inputValues[0], inputValues[1], inputValues[2])
        }
    }

    function addCustomCommand(name, description, command) {
        root.customCommands.push({ name: name, description: description, command: command })
    }

    function editCustomCommand(name, newDescription, newCommand) {
        root.customCommands = root.customCommands.map(function(command) {
            if (command.name === name) {
                return { name: name, description: newDescription, command: newCommand }
            }
            return command
        })
    }

    function removeCustomCommand(name) {
        root.customCommands = root.customCommands.filter(function(command) {
            return command.name !== name
        })
    }

    function resetModel() {
        commandList.model = []
    }

    function refreshModel() {
        if (root.hostId !== "") {
            commandList.model = LK.config.getCustomCommands(root.hostId).map(JSON.parse)
        }
    }
}