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
import "js/Test.js" as Test 

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

    /// For testing.
    function test() {
        return Test.test(root)
    }

    menuBar: MainMenuBar {
        onClickedAdd: {
            hostConfigurationDialog.hostId = ""
            hostConfigurationDialog.open()
        }
        onClickedRemove: {
            let hostId = hostTableModel.getSelectedHostId()
            LK.config.beginHostConfiguration()
            LK.config.removeHost(hostId)
            LK.config.endHostConfiguration()
            hostTableModel.toggleRow(hostTableModel.selectedRow)
            // LK.hosts retains host data until explicitly removed.
            LK.hosts.removeHost(hostId)
            LK.reload()
        }
        onClickedEdit: {
            LK.config.beginHostConfiguration()
            hostConfigurationDialog.hostId = hostTableModel.getSelectedHostId()
            hostConfigurationDialog.open()
        }
        onClickedCertificateMonitor: {
            certificateMonitorDialog.open()
        }
        onClickedPreferences: {
            preferencesDialog.open()
        }
        onClickedHotkeyHelp: {
            hotkeyHelp.open()
        }
        onFilterChanged: function(searchText) {
            hostTableModel.filter(searchText)
        }

        // Shortcuts are enabled if no host is selected.
        enableShortcuts: hostTableModel.selectedRow === -1
        enableEditButtons: hostTableModel.selectedRow !== -1
    }

    footer: StatusBar {
        id: statusBar
        visible: Theme.showStatusBar === true
        errorCount: root.errorCount
        jobsLeft: 0
    }

    Connections {
        target: LK

        function onReloaded(error) {
            hostTableModel.displayData = LK.hosts.getDisplayData()

            if (error !== "") {
                root.errorCount += 1;
                snackbarContainer.addSnackbar("Critical", error)
            }
        }
    }

    Connections {
        target: LK.hosts

        function onUpdateReceived(hostId) {
            hostTableModel.dataChangedForHost(hostId)
            hostTableModel.displayData = LK.hosts.getDisplayData()

            if (hostId === hostTableModel.getSelectedHostId()) {
                let jobsLeft = LK.hosts.getPendingCommandCount(hostId) + LK.hosts.getPendingMonitorCount(hostId)
                statusBar.jobsLeft = jobsLeft
                root.menuBar.refreshProgress = jobsLeft > 0 ? 0 : 100
            }
        }

        function onMonitorStateChanged(hostId, monitorId, newCriticality) {
            if (LK.config.getPreferences().showMonitorNotifications === true && ["Warning", "Error", "Critical"].includes(newCriticality)) {
                let title = hostId
                let message = `Monitor "${monitorId}" is now at level "${newCriticality}"`
                tray.showMessage(title, message)
            }

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
                LK.command.verifyHostKey(hostId, connectorId, keyId)
                LK.command.initializeHost(hostId)
            })
        }
    }

    Connections {
        target: LK.command

        function onConfirmationDialogOpened(text, buttonId, hostId, commandId, commandParams) {
            confirmationDialogLoader.setSource("./Dialog/ConfirmationDialog.qml", { text: text }) 
            confirmationDialogLoader.item.onAccepted.connect(() => LK.command.executeConfirmed(buttonId, hostId, commandId, commandParams))
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

        function onInputDialogOpened(inputSpecsJson, buttonId, hostId, commandId, commandParams) {
            let inputSpecs = JSON.parse(inputSpecsJson)

            inputDialog.inputSpecs = inputSpecs
            // TODO: need to clear previous connections?
            inputDialog.onInputValuesGiven.connect((inputValues) => {
                LK.command.executeConfirmed(buttonId, hostId, commandId, commandParams.concat(inputValues))
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
        LK.receiveUpdates()
        // Starts the thread that receives portal responses from D-Bus.
        DesktopPortal.receiveResponses()

        console.log("Current color palette: ", palette)

        if (LK.hosts.refresh_hosts_on_start()) {
            LK.command.forceInitializeHosts()
        }
    }


    Item {
        id: body
        objectName: "body"
        anchors.fill: parent
        property real splitSize: 0.0

        SplitView {
            anchors.fill: parent
            orientation: Qt.Vertical

            HostTable {
                id: hostTable
                objectName: "hostTable"
                width: parent.width
                SplitView.fillHeight: true

                model: HostTableModel {
                    id: hostTableModel
                    selectedRow: -1
                    displayData: LK.hosts.getDisplayData()

                    onSelectedRowChanged: {
                        detailsView.hostId = hostTableModel.getSelectedHostId()

                        if (detailsView.hostId !== "") {
                            if (!LK.hosts.isHostInitialized(detailsView.hostId)) {
                                LK.command.initializeHost(detailsView.hostId)
                            }
                            statusBar.jobsLeft = LK.hosts.getPendingCommandCount(detailsView.hostId) +
                                                 LK.hosts.getPendingMonitorCount(detailsView.hostId)
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
                objectName: "detailsView"
                visible: body.splitSize > 0.01
                width: parent.width
                hostId: hostTableModel.getSelectedHostId()

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
                    hostTableModel.toggleRow(hostTableModel.selectedRow)
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

        onConfigurationChanged: LK.reload()
    }

    CertificateMonitorDialog {
        id: certificateMonitorDialog
        anchors.centerIn: parent
        bottomMargin: 0.15 * parent.height
    }

    PreferencesDialog {
        id: preferencesDialog
        anchors.centerIn: parent
        bottomMargin: 0.15 * parent.height

        onConfigurationChanged: LK.reload()
    }

    // TODO: Repeater didn't work, figure out why.
    ConfigHelperDialog {
        groupName: "linux"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        groupName: "docker"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        groupName: "docker-compose"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        groupName: "systemd-service"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        groupName: "nixos"
        onConfigurationChanged: LK.reload()
    }

    CommandOutputDialog {
        id: commandOutputDialog
        property int pendingInvocation: 0

        enableShortcuts: visible
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

    LightkeeperTray {
        id: tray
        onShowClicked: root.toggleShow()
        onQuitClicked: root.quit()

        criticalCount: LK.hosts.monitorCriticalCount
        errorCount: LK.hosts.monitorErrorCount
        warningCount: LK.hosts.monitorWarningCount
        normalCount: LK.hosts.monitorNormalCount
        nodataCount: LK.hosts.monitorNoDataCount
    }

    function toggleShow() {
        if (root.visible) {
            root.hide()
        }
        else {
            root.show()
        }
    }

    function quit() {
        if (LK.config.getPreferences().closeToTray) {
            root.hide()
        }
        else {
            LK.stop()
            DesktopPortal.stop()
            Qt.quit()
        }
    }
}