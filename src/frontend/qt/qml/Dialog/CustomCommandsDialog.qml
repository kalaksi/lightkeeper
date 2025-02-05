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

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    title: `Custom commands`
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configSaved(string hostId)

    onOpened: {
        LK.config.beginHostConfiguration()
    }

    onAccepted: {
        LK.config.endHostConfiguration()
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
                model: LK.config.getCustomCommands(root.hostId).map(JSON.parse)

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
                        groupConfigDialog.groupName = root._selectedGroups[commandList.currentIndex]
                        groupConfigDialog.open()
                    }
                }

                ImageButton {
                    id: removeButton
                    imageSource: "qrc:/main/images/button/delete"
                    size: root.buttonSize
                    onClicked: {
                        let selectedGroup = root._selectedGroups[commandList.currentIndex]
                        LK.config.removeHostFromGroup(root.hostId, selectedGroup)
                        root.refreshGroups();
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
        width: parent.width
        height: 200
        onInputValuesGiven: function(inputValues) {
            LK.config.addCustomCommand(root.hostId, inputValues[0], inputValues[1], inputValues[2])
        }
    }
}