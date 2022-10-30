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

    implicitWidth: size * (commands.length + 0.5)
    implicitHeight: size

    signal clicked(string commandId)

    Row {
        anchors.right: parent.right

        Repeater {
            model: root.roundButtons === true ? root.commands : []

            RoundButton {
                onClicked: root.clicked(modelData.command_id)

                flat: root.flatButtons
                width: root.height
                height: width

                ToolTip.visible: hovered
                ToolTip.delay: 1000
                ToolTip.text: modelData.display_options.display_text

                Image {
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/" + modelData.display_options.display_icon
                    width: 0.85 * parent.width
                    height: width
                }
            }
        }

        Repeater {
            model: root.roundButtons === false ? root.commands : []

            Button {
                onClicked: root.clicked(modelData.command_id)

                flat: root.flatButtons
                width: root.height
                height: width

                ToolTip.visible: hovered
                ToolTip.delay: 1000
                ToolTip.text: modelData.display_options.display_text

                Image {
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/" + modelData.display_options.display_icon
                    width: 0.85 * parent.width
                    height: width
                }
            }
        }
    }
}