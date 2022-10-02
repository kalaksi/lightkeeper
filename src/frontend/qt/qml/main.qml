import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

import "js/CreateObject.js" as CreateObject

ApplicationWindow {
    id: root
    visible: true
    minimumWidth: 1024
    minimumHeight: 600
    width: 1280
    height: 768


    Material.theme: Material.System

    Component.onCompleted: {
        // Binding has to be done in a bit of a roundabout way here.
        lightkeeper_commands.onConfirmation_dialog_opened.connect((text, host_id, command_id, target_id) =>
            CreateObject.confirmationDialog(root, text, () => lightkeeper_commands.execute_confirmed(host_id, command_id, target_id))
        )
        lightkeeper_commands.onDetails_dialog_opened.connect(() => {
            let instanceId = CreateObject.detailsDialog(root, "", "", "")

            /* TODO: update dialog data
            lightkeeper_data.dataChanged.connect(() => {
                let data = lightkeeper_data.get_command_data(lightkeeper_data.get_selected_host())[0]
                if (typeof data !== "undefined") {
                    let instance = CreateObject.get("DetailsDialog", instanceId)

                    let commandResult = JSON.parse(data)
                    instance.text = commandResult.message
                    instance.errorText = commandResult.error
                    instance.criticality = commandResult.criticality
                }
            })
            */
        })
    }

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

                model: lightkeeper_data
            }

            HostDetails {
                id: details
                width: parent.width
                SplitView.minimumHeight: 0.5 * body.splitSize * parent.height
                SplitView.preferredHeight: body.splitSize * parent.height
                SplitView.maximumHeight: 1.5 * body.splitSize * parent.height

                model: lightkeeper_data
                commandsModel: lightkeeper_commands
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

        // Animations
        NumberAnimation {
            id: animateShowDetails
            target: body
            property: "splitSize"
            to: 0.66
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