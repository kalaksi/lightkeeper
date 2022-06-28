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
        id: body
        anchors.fill: parent
        property string selectedHost
        property bool showDetails: false
        property real splitSize: 0.0

        onShowDetailsChanged: function() {
            if (body.showDetails === true) {
                animateShowDetails.start()
            }
            else {
                animateHideDetails.start()
            }
        }

        PropertyAnimation {
            id: animateShowDetails
            target: body
            properties: "splitSize"
            to: 0.5
            duration: 150
        }

        PropertyAnimation {
            id: animateHideDetails
            target: body
            properties: "splitSize"
            to: 0.0
            duration: 150
        }

        states: State {
            name: "show_details"
            when: lightkeeper_data.selected_row >= 0

            PropertyChanges {
                target: body
                showDetails: true
            }
        }

        HostTable {
            id: table
            anchors.top: parent.top
            width: parent.width
            height: (1.0 - parent.splitSize) * parent.height
            ScrollBar.vertical: ScrollBar { }

            model: lightkeeper_data
        }

        HostDetails {
            id: details
            width: root.width
            height: parent.splitSize * parent.height
            anchors.bottom: parent.bottom

            model: lightkeeper_data
        }
    }
}