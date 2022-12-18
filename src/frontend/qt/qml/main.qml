import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11


ApplicationWindow {
    id: root
    visible: true
    minimumWidth: 1280
    minimumHeight: 768
    width: 1280
    height: 768

    property var _dialogInvocationIds: {}

    Material.theme: Material.System

    // TODO: when could Connection component be used instead of JS?
    Component.onCompleted: {
        _dialogInvocationIds = {}

        // Set up confirmation dialog on signal.
        _commandHandler.confirmation_dialog_opened.connect((text, host_id, command_id, target_id) => {
            confirmationDialogLoader.setSource("ConfirmationDialog.qml", { text: text }) 
            confirmationDialogLoader.item.onAccepted.connect(() => _commandHandler.execute_confirmed(host_id, command_id, [target_id]))
        })

        // Starts the thread that receives host state updates in the backend.
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

        _commandHandler.details_dialog_opened.connect((invocationId) => {
            let instanceId = detailsDialogManager.create()
            _dialogInvocationIds[invocationId] = instanceId
        })

        _commandHandler.details_subview_opened.connect((headerText, invocationId) => {
            hostDetails.openTextView(headerText, invocationId)
        })

        _commandHandler.logs_subview_opened.connect((headerText, invocationId) => {
            hostDetails.openLogView(headerText, invocationId)
        })

        _hostDataManager.command_result_received.connect((commandResultJson) => {
            let commandResult = JSON.parse(commandResultJson)

            if (commandResult.criticality !== "Normal") {
                toastLoader.setSource("Toast.qml", {
                    text: commandResult.message,
                    criticality: commandResult.criticality,
                })
            }

            let dialogInstanceId = _dialogInvocationIds[commandResult.invocation_id]
            if (typeof dialogInstanceId !== "undefined") {
                let dialog = detailsDialogManager.get(dialogInstanceId)
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
                onOpenInNewWindowClicked: (invocationId, text, errorText, criticality) => {
                    let instanceId = detailsDialogManager.create({
                        text: text,
                        errorText: errorText,
                        criticality: criticality,
                    })
                    root._dialogInvocationIds[invocationId] = instanceId
                }
                onCloseClicked: _hostTableModel.toggle_row(_hostTableModel.selected_row)
            }
        }

        // Dynamic component loaders
        Loader {
            id: confirmationDialogLoader
        }

        Loader {
            id: toastLoader
            anchors.bottom: body.bottom
            anchors.right: body.right
            anchors.margins: 20
        }

        DynamicObjectManager {
            id: detailsDialogManager

            DetailsDialog {
                y: root.y + 50
                x: root.x + 50
                width: root.width
                height: root.height
            }
        }

        // Animations
        NumberAnimation {
            id: animateMaximizeDetails
            target: body
            property: "splitSize"
            to: 1.0
            duration: 175
            easing.type: Easing.OutQuad
        }

        NumberAnimation {
            id: animateMinimizeDetails
            target: body
            property: "splitSize"
            to: 0.8
            duration: 175
            easing.type: Easing.OutQuad
        }

        NumberAnimation {
            id: animateShowDetails
            target: body
            property: "splitSize"
            to: 0.8
            duration: 175
            easing.type: Easing.OutQuad
        }

        NumberAnimation {
            id: animateHideDetails
            target: body
            property: "splitSize"
            to: 0.0
            duration: 175
            easing.type: Easing.OutQuad
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