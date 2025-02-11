import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../Button"
import "../Text"
import "../Misc"
import ".."

LightkeeperDialog {
    id: root
    property string hostId: ""
    property int buttonSize: 38
    property bool _loading: hostId === ""
    property var customCommands: LK.command.getCustomCommands(root.hostId).map(JSON.parse)

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    title: `Custom commands`
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged(string hostId)

    onOpened: {
        LK.config.beginHostConfiguration()
    }

    onAccepted: {
        LK.config.endHostConfiguration()
        root.configurationChanged(root.hostId)
    }

    onRejected: {
        LK.config.cancelHostConfiguration()
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
                model: root.customCommands
                modelPropertyName: "name"

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
                    property var currentCommand: root.customCommands[commandList.currentIndex]

                    imageSource: "qrc:/main/images/button/entry-edit"
                    size: root.buttonSize
                    onClicked: {
                        commandEditDialog.inputSpecs = [
                            { label: "Command name", field_type: "Text", default_value: currentCommand.name },
                            { label: "Description", field_type: "Text", default_value: currentCommand.description },
                            { label: "Shell command", field_type: "Text", default_value: currentCommand.command }
                        ]
                        commandEditDialog.open()
                    }
                }

                ImageButton {
                    id: removeButton
                    imageSource: "qrc:/main/images/button/delete"
                    size: root.buttonSize
                    onClicked: {
                        // TODO
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
        width: parent.width
        height: 200
        onInputValuesGiven: function(inputValues) {
            LK.config.addCustomCommand(root.hostId, inputValues[0], inputValues[1], inputValues[2])
        }
    }

    InputDialog {
        id: commandEditDialog
        title: "Edit command"
        width: parent.width
        height: 200
        onInputValuesGiven: function(inputValues) {
            LK.config.updateCustomCommand(root.hostId, inputValues[0], inputValues[1], inputValues[2])
        }
    }
}