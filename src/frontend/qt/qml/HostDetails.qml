import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    required property var model

    Rectangle {
        anchors.fill: parent
        color: Material.background

        Text {
            text: root.model.selected_row
        }
    }
}