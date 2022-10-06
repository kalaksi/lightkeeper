import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    required property string text
    property string color: "#555555"

    implicitWidth: parent.width
    implicitHeight: 35

    signal closeClicked()

    Rectangle {
        color: root.color
        anchors.fill: parent

        NormalText {
            anchors.verticalCenter: parent.verticalCenter
            leftPadding: 10
            text: root.text
            font.pointSize: 16
        }
    }

    Button {
        flat: true
        width: 0.8 * parent.height
        height: 0.8 * parent.height
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.margins: 5

        onClicked: root.closeClicked()

        Image {
            source: "qrc:/main/images/button/close"
            width: 0.5 * parent.width
            height: 0.8 * parent.height
            anchors.centerIn: parent

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: Material.foreground
                antialiasing: true
            }
        }
    }
}