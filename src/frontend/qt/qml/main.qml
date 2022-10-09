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

        _hostDataManager.update_received.connect((hostId) => {
            _hostTableModel.data_changed_for_host(hostId)

            if (hostId === hostDetails.hostId) {
                hostDetails.refresh()
            }
        })

        _hostDataManager.monitor_state_changed.connect((hostId, monitorId, newCriticality) => {
            hostTable.highlightMonitor(hostId, monitorId, newCriticality)
        })

        _hostTableModel.selected_row_changed.connect(() => {
            hostDetails.hostId = _hostTableModel.get_selected_host_id()
        })

        _hostTableModel.selection_activated.connect(() => {
            animateShowDetails.start()
        })

        _hostTableModel.selection_deactivated.connect(() => {
            animateHideDetails.start()
        })

        _commandHandler.confirmation_dialog_opened.connect((text, host_id, command_id, target_id) =>
            CreateObject.confirmationDialog(root, text, () => commands.execute_confirmed(host_id, command_id, target_id))
        )

        let _dialogInvocationIds = {}
        _commandHandler.text_dialog_opened.connect((invocationId) => {
            let instanceId = CreateObject.detailsDialog(root, "", "", "")
            _dialogInvocationIds[invocationId] = instanceId
        })

        _commandHandler.text_view_opened.connect((headerText, invocationId) => {
            hostDetails.openSubview(headerText)
        })

        _hostDataManager.command_result_received.connect((commandResultJson) => {
            let commandResult = JSON.parse(commandResultJson)

            let dialogInstanceId = _dialogInvocationIds[commandResult.invocation_id]
            
            if (typeof dialogInstanceId !== "undefined") {
                let dialog = CreateObject.get(dialogInstanceId)
                dialog.text = commandResult.message
                dialog.errorText = commandResult.error
                dialog.criticality = commandResult.criticality
            }
            else {
                hostDetails.refreshSubview(commandResult)
            }
        })
    }

    Item {
        id: body
        anchors.fill: parent
        property real splitSize: 0.0

        SplitView {
            anchors.fill: parent
            orientation: Qt.Vertical

            HostTable {
                id: hostTable
                width: parent.width
                SplitView.minimumWidth: body.width
                SplitView.fillHeight: true

                hostDataManager: _hostDataManager
                model: _hostTableModel
            }

            HostDetails {
                id: hostDetails
                width: parent.width
                SplitView.minimumHeight: 0.5 * body.splitSize * body.height
                SplitView.preferredHeight: body.splitSize * body.height
                SplitView.maximumHeight: 1.5 * body.splitSize * body.height

                hostId: _hostTableModel.get_selected_host_id()
                hostDataManager:_hostDataManager
                commandHandler: _commandHandler

                onMinimizeClicked: animateMinimizeDetails.start()
                onMaximizeClicked: animateMaximizeDetails.start()
                onCloseClicked: _hostTableModel.toggle_row(_hostTableModel.selected_row)
            }
        }

        // Animations
        NumberAnimation {
            id: animateMaximizeDetails
            target: body
            property: "splitSize"
            to: 1.0
            duration: 150
        }

        NumberAnimation {
            id: animateMinimizeDetails
            target: body
            property: "splitSize"
            to: 0.66
            duration: 150
        }

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
                    target: hostDetails 
                    visible: true
                }
            },
            State {
                name: "detailsHiddenVisibility"
                when: body.splitSize < 0.01

                PropertyChanges {
                    target: hostDetails
                    visible: false
                }
            }
        ]

    }
}