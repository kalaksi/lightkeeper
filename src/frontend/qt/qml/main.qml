import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import HostTableModel 1.0

import "./Dialog"
import "./Button"
import "./DetailsView"
import "./Misc"
import "js/Utils.js" as Utils

ApplicationWindow {
    property var _detailsDialogs: {}
    property int errorCount: 0

    id: root
    visible: true
    minimumWidth: 1400
    minimumHeight: 810
    width: minimumWidth + 100
    height: minimumHeight

    onWidthChanged: {
        hostTable.forceLayout()
    }

    onClosing: root.quit()

    menuBar: MainMenuBar {
        onClickedAdd: {
            hostConfigurationDialog.hostId = ""
            hostConfigurationDialog.open()
        }
        onClickedRemove: {
            ConfigManager.begin_host_configuration()
            ConfigManager.removeHost(_hostTableModel.getSelectedHostId())
            ConfigManager.end_host_configuration()
            reloadConfiguration()
        }
        onClickedEdit: {
            ConfigManager.begin_host_configuration()
            hostConfigurationDialog.hostId = _hostTableModel.getSelectedHostId()
            hostConfigurationDialog.open()
        }
        onClickedPreferences: {
            preferencesDialog.open()
        }
        onClickedHotkeyHelp: {
            hotkeyHelp.open()
        }
        onFilterChanged: function(searchText) {
            _hostTableModel.filter(searchText)
        }

        // Shortcuts are enabled if no host is selected.
        enableShortcuts: _hostTableModel.selectedRow === -1
    }

    footer: StatusBar {
        id: statusBar
        visible: Theme.showStatusBar === true
        errorCount: root.errorCount
        jobsLeft: 0
    }

    Connections {
        target: HostDataManager

        function onUpdateReceived(hostId) {
            _hostTableModel.dataChangedForHost(hostId)
            _hostTableModel.displayData = HostDataManager.getDisplayData()

            if (hostId === _hostTableModel.getSelectedHostId()) {
                statusBar.jobsLeft = HostDataManager.getPendingCommandCount(hostId) + HostDataManager.getPendingMonitorCount(hostId)
            }
        }

        function onHost_initialized(hostId) {
            let categories = CommandHandler.getAllHostCategories(hostId)
            for (const category of categories) {
                CommandHandler.refresh_monitors_of_category(hostId, category)
            }
        }

        function onHost_initialized_from_cache(hostId) {
            let categories = CommandHandler.getAllHostCategories(hostId)
            for (const category of categories) {
                CommandHandler.cached_refresh_monitors_of_category(hostId, category)
            }
        }

        function onMonitor_state_changed(hostId, monitorId, newCriticality) {
            hostTable.highlightMonitor(hostId, monitorId, newCriticality)
        }

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (["Error", "Critical"].includes(commandResult.criticality)) {
                root.errorCount += 1;
            }

            if (commandResult.show_in_notification === true &&
                (commandResult.criticality !== "Normal" || Theme.hide_info_notifications() === false)) {

                if (commandResult.error !== "") {
                    snackbarContainer.addSnackbar(commandResult.criticality, commandResult.error)
                }
                else {
                    snackbarContainer.addSnackbar(commandResult.criticality, commandResult.message)
                }
            }

            let dialogInstanceId = _detailsDialogs[invocationId]
            if (typeof dialogInstanceId !== "undefined") {
                let dialog = detailsDialogManager.get(dialogInstanceId)
                dialog.text = commandResult.message
                dialog.errorText = commandResult.error
                dialog.criticality = commandResult.criticality
            }
            else if (textDialog.pendingInvocation === invocationId) {
                textDialog.text = commandResult.message
            }
            else if (commandOutputDialog.pendingInvocation === invocationId) {
                commandOutputDialog.text = commandResult.message
                commandOutputDialog.progress = commandResult.progress
            }
        }

        function onErrorReceived(criticality, message) {
            root.errorCount += 1;
            if (criticality === "Critical") {
                // TODO: something better. This is not really an alert dialog.
                textDialog.text = message
                textDialog.open()
            }
            else {
                snackbarContainer.addSnackbar(criticality, message)
            }
        }

        function onVerificationRequested(hostId, connectorId, message, keyId) {
            let text = message + "\n\n" + keyId
            confirmationDialogLoader.setSource("./Dialog/ConfirmationDialog.qml", { text: text, }) 
            confirmationDialogLoader.item.onAccepted.connect(function() {
                CommandHandler.verifyHostKey(hostId, connectorId, keyId)
                CommandHandler.initializeHost(hostId)
            })
        }
    }

    Connections {
        target: CommandHandler

        function onConfirmationDialogOpened(text, hostId, commandId, commandParams) {
            confirmationDialogLoader.setSource("./Dialog/ConfirmationDialog.qml", { text: text }) 
            confirmationDialogLoader.item.onAccepted.connect(() => CommandHandler.executeConfirmed(hostId, commandId, commandParams))
        }

        function onDetailsDialogOpened(invocationId) {
            let instanceId = detailsDialogManager.create()
            _detailsDialogs[invocationId] = instanceId
        }

        function onTextDialogOpened(invocationId) {
            textDialog.pendingInvocation = invocationId
            textDialog.open()
        }

        function onCommandOutputDialogOpened(title, invocationId) {
            commandOutputDialog.pendingInvocation = invocationId
            commandOutputDialog.open()
        }

        function onInputDialogOpened(inputSpecsJson, hostId, commandId, commandParams) {
            let inputSpecs = JSON.parse(inputSpecsJson)

            inputDialog.inputSpecs = inputSpecs
            // TODO: need to clear previous connections?
            inputDialog.onInputValuesGiven.connect((inputValues) => {
                CommandHandler.executeConfirmed(hostId, commandId, commandParams.concat(inputValues))
            })
            inputDialog.open()
        }
    }

    Connections {
        target: DesktopPortal
        function onOpenFileResponse(token) {
            console.log("************ Received open file response: " + token)
            // TODO
        }

        function onError(message) {
            root.errorCount += 1;
            snackbarContainer.addSnackbar("Critical", message)
        }
    }

    Component.onCompleted: {
        _detailsDialogs = {}

        // Starts the thread that receives host state updates in the backend.
        HostDataManager.receive_updates()
        // Starts the thread that receives portal responses from D-Bus.
        DesktopPortal.receiveResponses()

        console.log("Current color palette: ", palette)

        if (HostDataManager.refresh_hosts_on_start()) {
            CommandHandler.forceInitializeHosts()
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
                SplitView.fillHeight: true

                model: HostTableModel {
                    id: _hostTableModel
                    selectedRow: -1
                    displayData: HostDataManager.getDisplayData()

                    onSelectedRowChanged: {
                        detailsView.hostId = _hostTableModel.getSelectedHostId()

                        if (detailsView.hostId !== "") {
                            if (!HostDataManager.is_host_initialized(detailsView.hostId)) {
                                CommandHandler.initializeHost(detailsView.hostId)
                            }
                            statusBar.jobsLeft = HostDataManager.getPendingCommandCount(detailsView.hostId) +
                                                HostDataManager.getPendingMonitorCount(detailsView.hostId)
                        }
                    }

                    onSelectionActivated: {
                        body.splitSize = 0.8
                    }

                    onSelectionDeactivated: {
                        body.splitSize = 0.0
                    }
                }
            }

            HostDetails {
                id: detailsView
                visible: body.splitSize > 0.01
                width: parent.width
                hostId: _hostTableModel.getSelectedHostId()

                SplitView.minimumHeight: 0.5 * body.splitSize * body.height
                SplitView.preferredHeight: body.splitSize * body.height
                SplitView.maximumHeight: 1.5 * body.splitSize * body.height

                onMinimizeClicked: {
                    body.splitSize = 0.8
                }
                onMaximizeClicked: {
                    body.splitSize = 1.0
                }
                onCloseClicked: {
                    _hostTableModel.toggleRow(_hostTableModel.selectedRow)
                }
            }
        }

        Behavior on splitSize {
            NumberAnimation {
                duration: Theme.animationDuration
                easing.type: Easing.OutQuad

                onFinished: {
                    // TODO: animate?
                    hostTable.centerRow()
                }
            }
        }
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
        anchors.centerIn: parent
    }

    HostConfigurationDialog {
        id: hostConfigurationDialog
        anchors.centerIn: parent
        bottomMargin: 0.15 * parent.height

        onConfigurationChanged: {
            reloadConfiguration()
        }
    }

    PreferencesDialog {
        id: preferencesDialog
        anchors.centerIn: parent
        bottomMargin: 0.15 * parent.height

        onConfigurationChanged: {
            reloadConfiguration()
        }
    }

    // TODO: Repeater didn't work, figure out why.
    ConfigHelperDialog {
        groupName: "linux"
        onConfigurationChanged: reloadConfiguration()
    }

    ConfigHelperDialog {
        groupName: "docker"
        onConfigurationChanged: reloadConfiguration()
    }

    ConfigHelperDialog {
        groupName: "docker-compose"
        onConfigurationChanged: reloadConfiguration()
    }

    ConfigHelperDialog {
        groupName: "systemd-service"
        onConfigurationChanged: reloadConfiguration()
    }

    ConfigHelperDialog {
        groupName: "nixos"
        onConfigurationChanged: reloadConfiguration()
    }

    CommandOutputDialog {
        id: commandOutputDialog
        property int pendingInvocation: 0

        anchors.centerIn: parent
        width: root.width * 0.6
        height: root.height * 0.8
    }

    TextDialog {
        id: textDialog
        property int pendingInvocation: 0

        anchors.centerIn: parent
        width: Utils.clamp(implicitWidth, root.width * 0.5, root.width * 0.8)
        height: Utils.clamp(implicitHeight, root.height * 0.5, root.height * 0.8)
    }

    HotkeyHelp {
        id: hotkeyHelp
        anchors.centerIn: parent
        height: Utils.clamp(implicitHeight, root.height * 0.5, root.height * 0.8)
    }

    Shortcut {
        sequence: StandardKey.Quit
        onActivated: root.quit()
    }


    function reloadConfiguration() {
        HostDataManager.reset()
        _hostTableModel.toggleRow(_hostTableModel.selectedRow)
        _hostTableModel.displayData = HostDataManager.getDisplayData()

        let configs = ConfigManager.reloadConfiguration()
        CommandHandler.reconfigure(configs[0], configs[1])
    }

    function quit() {
        CommandHandler.stop()
        HostDataManager.stop()
        DesktopPortal.stop()
        Qt.quit()
    }
}