import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

import HostTableModel 1.0

import "./Dialog"
import "./Button"
import "./DetailsView"
import "js/Utils.js" as Utils

ApplicationWindow {
    id: root
    visible: true
    minimumWidth: 1400
    minimumHeight: 800
    width: 1400
    height: 800

    property var _detailsDialogs: {}
    property int _textDialogPendingInvocation: 0

    header: ToolBar {
        RowLayout {
            anchors.fill: parent

            // Spacer
            Item {
                Layout.fillWidth: true
            }

            ToolButton {
                icon.source: "qrc:/main/images/button/add"
                onClicked: {
                    hostConfigurationDialog.hostId = ""
                    hostConfigurationDialog.open()
                }
            }

            ToolButton {
                enabled: _hostTableModel.selected_row >= 0
                opacity: Theme.opacity(enabled)
                icon.source: "qrc:/main/images/button/entry-edit"
                onClicked: {
                    ConfigManager.begin_host_configuration()
                    hostConfigurationDialog.hostId = _hostTableModel.get_selected_host_id()
                    hostConfigurationDialog.open()
                }
            }

            ToolButton {
                enabled: _hostTableModel.selected_row >= 0
                opacity: Theme.opacity(enabled)
                icon.source: "qrc:/main/images/button/remove"
                onClicked: {
                    ConfigManager.begin_host_configuration()
                    ConfigManager.remove_host(_hostTableModel.get_selected_host_id())
                    ConfigManager.end_host_configuration()
                }
            }
        }
    }

    Material.theme: Material.Dark

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

            if (commandResult.show_in_notification === true &&
                (commandResult.criticality !== "Normal" || Theme.hide_info_notifications() === false)) {

                snackbarContainer.addSnackbar(commandResult.criticality, commandResult.message)
            }

            let dialogInstanceId = _detailsDialogs[commandResult.invocation_id]
            if (typeof dialogInstanceId !== "undefined") {
                let dialog = detailsDialogManager.get(dialogInstanceId)
                dialog.text = commandResult.message
                dialog.errorText = commandResult.error
                dialog.criticality = commandResult.criticality
            }
            else if (_textDialogPendingInvocation === commandResult.invocation_id) {
                textDialog.text = commandResult.message
            }
            else {
                detailsView.refreshSubview(commandResult)
            }
        }

        function onError_received(criticality, message) {
            if (criticality === "Critical") {
                // TODO: something better. This is not really an alert dialog.
                textDialog.text = message
                textDialog.open()
            }
            else {
                snackbarContainer.addSnackbar(criticality, message)
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
            _detailsDialogs[invocationId] = instanceId
        }

        function onText_dialog_opened(invocationId) {
            _textDialogPendingInvocation = invocationId
            textDialog.open()
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
            body.splitSize = 0.8
        }

        function onSelection_deactivated() {
            body.splitSize = 0.0
        }
    }


    Component.onCompleted: {
        _detailsDialogs = {}

        // Starts the thread that receives host state updates in the backend.
        HostDataManager.receive_updates()

        if (HostDataManager.refresh_hosts_on_start()) {
            CommandHandler.force_initialize_hosts()
        }
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

                onMinimizeClicked: {
                    body.splitSize = 0.8
                }
                onMaximizeClicked: {
                    body.splitSize = 1.0
                }
                onOpenInNewWindowClicked: (invocationId, text, errorText, criticality) => {
                    let instanceId = detailsDialogManager.create({
                        text: text,
                        errorText: errorText,
                        criticality: criticality,
                    })
                    root._detailsDialogs[invocationId] = instanceId
                }
                onCloseClicked: _hostTableModel.toggle_row(_hostTableModel.selected_row)
            }
        }

        // Animations
        Behavior on splitSize {
            NumberAnimation {
                duration: Theme.animation_duration()
                easing.type: Easing.OutQuad

                onFinished: {
                    // TODO: animate?
                    hostTable.centerRow()
                }
            }
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

    // Dynamic component loaders
    Loader {
        id: confirmationDialogLoader
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

    SnackbarContainer {
        id: snackbarContainer
        anchors.fill: parent
        anchors.margins: 20
    }


    // Modal dialogs
    InputDialog {
        id: inputDialog
        visible: false
        anchors.centerIn: parent
    }

    HostConfigurationDialog {
        id: hostConfigurationDialog
        visible: false
        anchors.centerIn: parent
        bottomMargin: 0.12 * parent.height
    }

    TextDialog {
        id: textDialog
        visible: false
        anchors.centerIn: parent
        width: Utils.clamp(implicitWidth, root.width * 0.5, root.width * 0.8)
        height: Utils.clamp(implicitHeight, root.height * 0.5, root.height * 0.8)
    }
}