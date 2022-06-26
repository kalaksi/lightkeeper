import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

ApplicationWindow {
    id: root_window
    visible: true
    minimumWidth: 800
    minimumHeight: 400
    width: 1024
    height: 600

    Material.theme: Material.Light

    Item {
        anchors.fill: parent

        HostTable {
            anchors.fill: parent
            model: lightkeeper_data
        }
    }

}