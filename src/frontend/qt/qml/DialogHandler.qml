/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound
import QtQuick

import "./Dialog"
import "js/Utils.js" as Utils


Item {
    id: root
    anchors.fill: parent

    InputDialog {
        id: inputDialog
    }

    HostConfigurationDialog {
        id: hostConfigDialog
        bottomMargin: 0.13 * parent.height

        onConfigurationChanged: LK.reload()
    }

    CertificateMonitorDialog {
        id: certificateMonitorDialog
        bottomMargin: 0.15 * parent.height
    }

    PreferencesDialog {
        id: preferencesDialog
        bottomMargin: 0.10 * parent.height

        onConfigurationChanged: LK.reload()
    }

    HotkeyHelp {
        id: hotkeyHelp
        height: Utils.clamp(implicitHeight, root.height * 0.5, root.height * 0.8)
    }

    CustomCommandsDialog {
        id: customCommandsDialog
        bottomMargin: 0.15 * parent.height

        // TODO: don't force full reload
        onConfigurationChanged: LK.reload()
    }

    DynamicObjectManager {
        id: confirmationDialogManager

        ConfirmationDialog {
            parent: root
        }
    }

    CommandOutputDialog {
        id: commandOutputDialog

        enableShortcuts: visible
        width: root.width * 0.6
        height: root.height * 0.8

        onMoveToTab: function(invocationId, title, text, errorText, progress) {
            LK.command.commandOutputViewOpened(invocationId, title, text, errorText, progress)
        }
    }

    TextDialog {
        id: textDialog
        property int pendingInvocation: 0

        width: Utils.clamp(implicitWidth, root.width * 0.5, root.width * 0.8)
        height: Utils.clamp(implicitHeight, root.height * 0.5, root.height * 0.8)
    }

    // TODO: something better? This is not really an alert dialog.
    TextDialog {
        id: errorDialog

        width: Utils.clamp(implicitWidth, root.width * 0.5, root.width * 0.8)
        height: Utils.clamp(implicitHeight, root.height * 0.5, root.height * 0.8)
    }

    // NOTE: ConfigHelperDialogs are curretly displayed automatically when needed. See ConfigHelperDialog.qml.
    // Currently has some code duplication here which could be improved.
    ConfigHelperDialog {
        parent: root
        groupName: "linux"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        parent: root
        groupName: "docker"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        parent: root
        groupName: "docker-compose"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        parent: root
        groupName: "systemd-service"
        onConfigurationChanged: LK.reload()
    }

    ConfigHelperDialog {
        parent: root
        groupName: "nixos"
        onConfigurationChanged: LK.reload()
    }


    function openInput(inputSpecs, onInputValuesGiven) {
        inputDialog.inputSpecs = inputSpecs

        // Removes connection after triggering once.
        var connectOnce = function(inputValues) {
            inputDialog.inputValuesGiven.disconnect(connectOnce)
            onInputValuesGiven(inputValues)
        }

        inputDialog.inputValuesGiven.connect(connectOnce)
        inputDialog.rejected.connect(() => {
            inputDialog.inputValuesGiven.disconnect(connectOnce)
        })
        inputDialog.open()
    }

    function openNewHostConfig(hostId) {
        hostConfigDialog.hostId = ""
        hostConfigDialog.open()
    }

    function openHostConfig(hostId) {
        hostConfigDialog.hostId = hostId
        hostConfigDialog.open()
    }

    function openCertificateMonitor() {
        certificateMonitorDialog.open()
    }

    function openPreferences() {
        preferencesDialog.open()
    }

    function openHotkeyHelp() {
        hotkeyHelp.open()
    }

    function openTextDialog(invocationId) {
        textDialog.pendingInvocation = invocationId
        textDialog.open()
    }

    function updateTextDialog(invocationId, text) {
        if (textDialog.visible === false || textDialog.pendingInvocation !== invocationId) {
            return
        }

        textDialog.text = text
    }

    function openErrorDialog(text) {
        errorDialog.text = text
        errorDialog.open()
    }

    function openCommandOutputDialog(invocationId, title) {
        commandOutputDialog.title = title
        commandOutputDialog.pendingInvocation = invocationId
        commandOutputDialog.open()
    }

    function updateCommandOutputDialog(invocationId, text, errorText, progress) {
        if (commandOutputDialog.visible === false || commandOutputDialog.pendingInvocation !== invocationId) {
            return
        }

        // Contains incremental update.
        // Update last line too since it could be incomplete. It's empty if it's not.
        let lastNewlineIndex = commandOutputDialog.text.lastIndexOf("\n")
        commandOutputDialog.text = lastNewlineIndex === -1 ?
            text :
            commandOutputDialog.text.substring(0, lastNewlineIndex + 1) + text

        commandOutputDialog.errorText = errorText
        commandOutputDialog.progress = progress
    }

    function openConfirmationDialog(text, onAccepted, acceptOnly = false) {
        let [instanceId, instance] = confirmationDialogManager.create(
            {
                text: text,
                acceptOnly: acceptOnly
            },
            // Removes connection after triggering once.
            {
                onAccepted: () => {
                    confirmationDialogManager.destroyInstance(instanceId)
                    onAccepted()
                },
                onRejected: () => {
                    confirmationDialogManager.destroyInstance(instanceId)
                }
            }
        )
    }

    function openCustomCommandsDialog(hostId) {
        customCommandsDialog.hostId = hostId
        customCommandsDialog.open()
    }
}
