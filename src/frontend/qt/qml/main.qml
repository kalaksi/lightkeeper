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
        _hostDataManager.receive_updates()

        _hostDataManager.onUpdate_received.connect((hostId) => {
            _hostTableModel.data_changed_for_host(hostId)

            if (hostId === _hostTableModel.get_selected_host_id()) {
                body.selectedHostId = hostId
            }
        })

/*
        _hostTableModel.onSelected_row_changed(() => {
            body.selectedHostId = _hostTableModel.get_selected_host_id()
        })
        */

        _commandHandler.onConfirmation_dialog_opened.connect((text, host_id, command_id, target_id) =>
            CreateObject.confirmationDialog(root, text, () => commands.execute_confirmed(host_id, command_id, target_id))
        )
        _commandHandler.onDetails_dialog_opened.connect(() => {
            let instanceId = CreateObject.detailsDialog(root, "", "", "")

            /* TODO: update dialog data
            data.dataChanged.connect(() => {
                let data = data.get_command_data(data.get_selected_host())[0]
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
        property string selectedHostId: ""
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

                hostDataManager: _hostDataManager
                model: _hostTableModel
            }

            HostDetails {
                id: details
                width: parent.width
                SplitView.minimumHeight: 0.5 * body.splitSize * parent.height
                SplitView.preferredHeight: body.splitSize * parent.height
                SplitView.maximumHeight: 1.5 * body.splitSize * parent.height

                hostId: body.selectedHostId
                hostDataManager:_hostDataManager
                commandHandler: _commandHandler
            }
        }

        // Slots and bindings
        onShowDetailsChanged: function() {
            if (showDetails === true) {
                body.selectedHostId = _hostTableModel.get_selected_host_id()
                animateShowDetails.start()
            }
            else {
                animateHideDetails.start()
            }
        }

        Binding { 
            target: body
            property: "showDetails"
            value: _hostTableModel.selected_row >= 0
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