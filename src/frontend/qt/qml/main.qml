import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

ApplicationWindow {
    id: root
    visible: true
    minimumWidth: 800
    minimumHeight: 400
    width: 1024
    height: 600

    Material.theme: Material.Light

    Item {
        anchors.fill: parent
        property string selectedHost

        HostTable {
            id: table
            anchors.top: parent.top
            width: parent.width
            height: 0.5 * parent.height
            ScrollBar.vertical: ScrollBar { }

            model: lightkeeper_data

        }

        HostDetails {
            width: root.width
            height: 0.5 * parent.height
            anchors.bottom: parent.bottom
            visible: true

            model: lightkeeper_data
            selectedHost: "test"
        }
    }

}