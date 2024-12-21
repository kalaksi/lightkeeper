import QtQuick
import QtQuick.Controls
import QtQuick.Layouts


CategoryGroupBox {
    id: root

    label: GroupBoxLabel {
        id: groupBoxLabel
        anchors.left: root.left
        anchors.right: root.right

        text: "Custom commands"
        icon: "qrc:///main/images/button/terminal"
        color: "#252525"
        border: 1
        borderColor: "#656565"
        showConfigButton: true
    }

    ColumnLayout {
        id: column
        anchors.fill: parent
        spacing: 0

        Item {
            id: customCommands
            width: parent.width
            height: 90

            // Background.
            Rectangle {
                anchors.fill: parent
                color: "#50808080"
            }
        }
    }
}