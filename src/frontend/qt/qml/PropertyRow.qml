import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    implicitWidth: parent.width
    implicitHeight: label.height
    property alias label: label.text
    property alias value: value.text

    RowLayout {
        anchors.fill: parent

        Label {
            id: label
            text: ""
            horizontalAlignment: Text.AlignRight
            Layout.preferredWidth: 0.5 * parent.width
            Layout.alignment: Qt.AlignLeft
        }

        Text {
            text: ":"
            color: Qt.darker(Material.foreground, 1.5)
            horizontalAlignment: Text.AlignLeft
            Layout.preferredWidth: 10
            Layout.alignment: Qt.AlignLeft
        }

        Text {
            id: value
            text: ""
            color: Material.foreground
            horizontalAlignment: Text.AlignLeft
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignRight
        }
    }
}