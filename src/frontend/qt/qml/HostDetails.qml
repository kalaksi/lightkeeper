import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    required property var model
    required property string selectedHost

    Rectangle {
        anchors.fill: parent
        color: Material.background

        Text {
            text: selectedHost
        }
    }
}