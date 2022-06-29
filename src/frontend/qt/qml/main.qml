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

    Material.theme: Material.System

    Item {
        id: body
        anchors.fill: parent
        property real splitSize: 0.0
        property bool showDetails: false

        SplitView {
            anchors.fill: parent
            orientation: Qt.Vertical

            HostTable {
                id: table
                width: parent.width
                SplitView.minimumWidth: body.width
                SplitView.fillHeight: true
                ScrollBar.vertical: ScrollBar { }

                model: lightkeeper_data
            }

            HostDetails {
                id: details
                width: parent.width
                SplitView.minimumHeight: 0.5 * body.splitSize * parent.height
                SplitView.preferredHeight: body.splitSize * parent.height
                SplitView.maximumHeight: 1.5 * body.splitSize * parent.height

                model: lightkeeper_data
            }
        }

        onShowDetailsChanged: function() {
            if (showDetails === true) {
                animateShowDetails.start()
            }
            else {
                animateHideDetails.start()
            }
        }

        NumberAnimation {
            id: animateShowDetails
            target: body
            property: "splitSize"
            to: 0.5
            duration: 150
        }

        NumberAnimation {
            id: animateHideDetails
            target: body
            property: "splitSize"
            to: 0.0
            duration: 150
        }

        Binding { 
            target: body
            property: "showDetails"
            value: lightkeeper_data.selected_row >= 0
        }

        states: [
            State {
                name: "detailsShownVisibility"
                when: body.splitSize > 0.01

                PropertyChanges {
                    target: details
                    visible: true
                }
            },
            State {
                name: "detailsHiddenVisibility"
                when: body.splitSize < 0.01

                PropertyChanges {
                    target: details
                    visible: false
                }
            }
        ]

    }
}