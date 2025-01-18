import QtQuick
import QtQuick.Window
import Qt.labs.qmlmodels
import QtQuick.Controls
import QtQuick.Layouts

import HostTableModel

import "./Button"
import "./DetailsView"
import "./Misc"
import "js/Utils.js" as Utils
import "js/Test.js" as Test 


ApplicationWindow {
    property int errorCount: 0
    property alias dialogHandler: dialogHandlerLoader.item

    id: root
    visible: true
    minimumWidth: 1400
    minimumHeight: 810
    width: minimumWidth + 100
    height: minimumHeight
    font.pointSize: Theme.fontSize
    title: "Lightkeeper (beta)"

    palette.alternateBase: Theme.alternateBaseColor
    palette.base: Theme.baseColor
    palette.brightText: "#ffffff"
    palette.button: "#31363b"
    palette.buttonText: Theme.textColor
    palette.dark: "#141618"
    palette.highlight: "#3daee9"
    palette.highlightedText: "#fcfcfc"
    palette.light: "#474d54"
    palette.link: "#1d99f3"
    palette.linkVisited: "#9b59b6"
    palette.mid: "#24282b"
    palette.midlight: "#3a4045"
    palette.shadow: "#0f1012"
    palette.text: "#fcfcfc"
    palette.toolTipBase: "#31363b"
    palette.toolTipText: "#fcfcfc"
    palette.window: "#2a2e32"
    palette.windowText: Theme.textColor


    onWidthChanged: {
        hostTable.forceLayout()
    }

    onClosing: function(close) {
        if (LK.config.getPreferences().closeToTray) {
            close.accepted = false
            root.hide()
        }
        else {
            root.quit()
        }
    }


    /// For testing.
    function test() {
        return Test.test(root)
    }

    menuBar: MainMenuBar {
        onClickedAdd: {
            dialogHandler.openNewHostConfig()
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
            let hostId = hostTableModel.getSelectedHostId()
            dialogHandler.openHostConfig(hostId)
        }
        onClickedCertificateMonitor: {
            dialogHandler.openCertificateMonitor()
        }
        onClickedPreferences: {
            dialogHandler.openPreferences()
        }
        onClickedHotkeyHelp: {
            dialogHandler.openHotkeyHelp()
        }
        onFilterChanged: function(searchText) {
            hostTableModel.filter(searchText)
        }

        onHotReload: function() {
            dialogHandlerLoader.source = ""
            dialogHandlerLoader.active = false
            dialogHandlerLoader.source = "DialogHandler.qml"
            dialogHandlerLoader.active = true
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

            // No need to check if invocation is relevant to specific dialogs. DialogHandler takes care of that.
            dialogHandler.updateTextDialog(invocationId, commandResult.message)
            dialogHandler.updateCommandOutputDialog(invocationId, commandResult.message, commandResult.progress)
        }

        function onErrorReceived(criticality, message) {
            root.errorCount += 1;
            if (criticality === "Critical") {
                dialogHandler.openErrorDialog(message)
            }
            else {
                snackbarContainer.addSnackbar(criticality, message)
            }
        }

        function onVerificationRequested(hostId, connectorId, message, keyId) {
            let text = message + "\n\n" + keyId

            dialogHandler.openConfirmationDialog(
                text,
                () => {
                    LK.command.verifyHostKey(hostId, connectorId, keyId)
                    LK.command.initializeHost(hostId)
                }
            )
        }
    }

    Connections {
        target: LK.command

        function onConfirmationDialogOpened(text, buttonId, hostId, commandId, commandParams) {
            dialogHandler.openConfirmationDialog(text, () => LK.command.executeConfirmed(buttonId, hostId, commandId, commandParams))
        }

        function onTextDialogOpened(invocationId) {
            dialogHandler.openTextDialog(invocationId)
        }

        function onCommandOutputDialogOpened(title, invocationId) {
            dialogHandler.openCommandOutputDialog(invocationId, title)
        }

        function onInputDialogOpened(inputSpecsJson, buttonId, hostId, commandId, commandParams) {
            let inputSpecs = JSON.parse(inputSpecsJson)
            dialogHandler.openInput(
                inputSpecs,
                (inputValues) => LK.command.executeConfirmed(buttonId, hostId, commandId, commandParams.concat(inputValues))
            )
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
        // Starts the thread that receives host state updates in the backend.
        LK.receiveUpdates()
        // Starts the thread that receives portal responses from D-Bus.
        DesktopPortal.receiveResponses()

        if (LK.hosts.refresh_hosts_on_start()) {
            LK.command.forceInitializeHosts()
        }
    }

    Loader {
        id: dialogHandlerLoader
        anchors.fill: parent
        source: "DialogHandler.qml"
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
                onCustomCommandsDialogOpened: {
                    dialogHandler.openCustomCommandsDialog(detailsView.hostId)
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

    SnackbarContainer {
        id: snackbarContainer
        anchors.fill: parent
        anchors.margins: 20
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
        LK.stop()
        DesktopPortal.stop()
        Qt.quit()
    }

    function reload() {
        // todo
    }
}