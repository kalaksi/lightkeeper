import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

import HostTableModel 1.0

import "./Dialog"
import "./DetailsView"

ApplicationWindow {
    id: root
    visible: true
    minimumWidth: 1400
    minimumHeight: 800
    width: 1400
    height: 800

    property var _dialogInvocationIds: {}

    Material.theme: Material.System

    Connections {
        target: HostDataManager

        function onUpdate_received(hostId) {
            _hostTableModel.data_changed_for_host(hostId)

            if (hostId === detailsView.hostId) {
                detailsView.refresh()
            }
        }

        function onHost_initialized(hostId) {
            let categories = CommandHandler.get_all_host_categories(hostId)
            for (const category of categories) {
                let invocation_ids = CommandHandler.refresh_monitors_of_category(hostId, category)
                HostDataManager.add_pending_monitor_invocations(hostId, category, invocation_ids)
            }
        }

        function onHost_initialized_from_cache(hostId) {
            let categories = CommandHandler.get_all_host_categories(hostId)
            for (const category of categories) {
                let invocation_ids = CommandHandler.cached_refresh_monitors_of_category(hostId, category)
                HostDataManager.add_pending_monitor_invocations(hostId, category, invocation_ids)
            }
        }

        function onMonitor_state_changed(hostId, monitorId, newCriticality) {
            hostTable.highlightMonitor(hostId, monitorId, newCriticality)
        }

        function onCommand_result_received(commandResultJson) {
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
                detailsView.refreshSubview(commandResult)
            }
        }
    }

    Connections {
        target: CommandHandler

        // Set up confirmation dialog on signal.
        function onConfirmation_dialog_opened(text, hostId, commandId, commandParams) {
            confirmationDialogLoader.setSource("./Dialog/ConfirmationDialog.qml", { text: text }) 
            confirmationDialogLoader.item.onAccepted.connect(() => CommandHandler.execute_confirmed(hostId, commandId, commandParams))
        }

        function onDetails_dialog_opened(invocationId) {
            let instanceId = detailsDialogManager.create()
            _dialogInvocationIds[invocationId] = instanceId
        }

        function onInput_dialog_opened(input_specs_json, hostId, commandId, commandParams) {
            let inputSpecs = JSON.parse(input_specs_json)

            inputDialog.inputSpecs = inputSpecs
            inputDialog.onInputValuesGiven.connect((inputValues) => {
                CommandHandler.execute_confirmed(hostId, commandId, commandParams.concat(inputValues))
            })
            inputDialog.open()
        }
    }

    Connections {
        target: _hostTableModel

        function onSelected_row_changed() {
            detailsView.hostId = _hostTableModel.get_selected_host_id()

            if (detailsView.hostId !== "") {
                if (!HostDataManager.is_host_initialized(detailsView.hostId)) {
                    CommandHandler.initialize_host(detailsView.hostId)
                }
            }
        }

        function onSelection_activated() {
            animateShowDetails.start()
        }

        function onSelection_deactivated() {
            animateHideDetails.start()
        }
    }


    Component.onCompleted: {
        _dialogInvocationIds = {}

        // Starts the thread that receives host state updates in the backend.
        HostDataManager.receive_updates()
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

                model: HostTableModel {
                    id: _hostTableModel
                    display_data: HostDataManager.get_display_data()
                }

            }

            HostDetails {
                id: detailsView
                width: parent.width
                SplitView.minimumHeight: 0.5 * body.splitSize * body.height
                SplitView.preferredHeight: body.splitSize * body.height
                SplitView.maximumHeight: 1.5 * body.splitSize * body.height

                hostId: _hostTableModel.get_selected_host_id()

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
                    target: detailsView 
                    visible: true
                }
            },
            State {
                name: "detailsHiddenVisibility"
                when: body.splitSize < 0.01

                PropertyChanges {
                    target: detailsView
                    visible: false
                }
            }
        ]
    }

    InputDialog {
        id: inputDialog
        anchors.centerIn: parent
        visible: false
    }
}