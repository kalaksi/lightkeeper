import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15


Item {
    id: root
    property var commands: []
    property int size: 24
    property bool flatButtons: true
    property bool roundButtons: true
    property bool collapsed: false

    implicitWidth: size * (commands.length + 0.5)
    implicitHeight: size

    signal clicked(string commandId)

    Row {
        anchors.right: parent.right

        Repeater {
            model: root.commands

            ImageButton {
                width: root.size
                height: root.size
                flatButton: root.flatButtons
                roundButton: root.roundButtons
                tooltip: modelData.display_options.display_text
                imageSource: "qrc:/main/images/button/" + modelData.display_options.display_icon
                onClicked: root.clicked(modelData.command_id)
            }
        }
    }
}