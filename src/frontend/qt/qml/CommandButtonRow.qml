import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15


Item {
    id: root
    property var commands: []
    property int size: 24

    implicitWidth: size * (commands.length + 0.5)
    implicitHeight: size

    signal clicked(string commandId)

    Row {
        anchors.right: parent.right

        Repeater {
            model: root.commands

            RoundButton {
                id: button
                onClicked: root.clicked(modelData.command_id)

                flat: true
                width: root.height
                height: width

                ToolTip {
                    id: toolTip
                    text: modelData.display_options.display_text
                    visible: text.length > 0 && mouseArea.containsMouse
                }

                Image {
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/" + modelData.display_options.display_icon
                    width: 0.85 * parent.width
                    height: width
                }

                MouseArea {
                    id: mouseArea
                    anchors.fill: parent
                    hoverEnabled: true
                }
            }
        }
    }
}