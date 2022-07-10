import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15


Item {
    id: root
    property var commands: []

    signal clicked(string commandId, string subcommand)

    RowLayout {
        anchors.fill: parent

        Repeater {
            model: root.commands

            Repeater {
                property var command: modelData

                model: modelData.subcommands

                RoundButton {
                    id: button
                    property real scale: 0.3
                    onClicked: root.clicked(command.command_id, modelData.subcommand)

                    flat: true
                    width: scale * parent.height
                    height: scale * parent.height

                    Image {
                        source: "qrc:/main/images/button/" + modelData.display_icon
                        width: parent.width
                        height: parent.height
                    }
                }
            }
        }
    }
}