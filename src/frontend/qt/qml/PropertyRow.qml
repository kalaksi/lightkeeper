import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root

    property string targetId
    property string hostId: ""
    property var rowCommands: []
    property var commandsModel
    
    property alias label: label.text
    property alias value: value.text

    implicitWidth: parent.width
    implicitHeight: label.height

    Row {
        anchors.fill: parent

        Item {
            id: labelAndValue
            implicitWidth: root.implicitWidth
            implicitHeight: root.implicitHeight

            Label {
                id: label
                text: ""
                anchors.left: parent.left
            }

            Text {
                id: value
                text: ""
                color: Material.foreground
                anchors.right: commands.left
            }

            // Row-level command buttons.
            CommandButtonRow {
                id: commands

                anchors.right: parent.right
                implicitWidth: 24 * root.rowCommands.length
                implicitHeight: parent.height

                commands: root.rowCommands
                onClicked: function(commandId) {
                    root.commandsModel.execute(root.hostId, commandId, root.targetId)
                }
            }
        }

    }
}