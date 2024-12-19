import QtQuick
import QtQuick.Controls
import QtQuick.Layouts


CategoryGroupBox {
    id: root

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