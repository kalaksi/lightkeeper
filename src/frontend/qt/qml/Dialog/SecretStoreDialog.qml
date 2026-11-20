/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Text"
import "../StyleOverride"
import ".."


LightkeeperDialog {
    id: root
    property string storeStatus: ""
    property bool _busy: false
    property string _errorText: ""
    /// Skip clearing fields when reopening after Ok (Dialog closes before async finishes).
    property bool _preserveOnOpen: false

    readonly property bool locked: root.storeStatus === "Locked"
    readonly property bool needsSetup: root.storeStatus === "Uninitialized" || root.storeStatus === "NoKey"
    readonly property bool isProtected: root.storeStatus === "UnlockedWithPassphrase"
    readonly property string _passphraseHint: root.needsSetup ? "Leave empty for no passphrase protection."
        : root.isProtected ? "Leave empty to remove protection."
        : ""
    readonly property string _okText: root._busy ? (root.locked ? "Unlocking..." : "Working...")
        : root.locked ? "Unlock"
        : root.storeStatus === "NoKey" ? "Re-initialize"
        : root.storeStatus === "Uninitialized" ? "Set up"
        : root.isProtected ? "Change passphrase"
        : "Set passphrase"

    title: root.locked ? "Unlock Lightkeeper's secret store"
        : root.storeStatus === "NoKey" ? "Re-initialize Lightkeeper's secret store"
        : root.storeStatus === "Uninitialized" ? "Set up Lightkeeper Core"
        : root.isProtected ? "Change Lightkeeper's secret store passphrase"
        : "Set new passphrase"
    modal: true
    implicitWidth: 520
    implicitHeight: rootColumn.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    onOpened: {
        if (!root._preserveOnOpen) {
            root.storeStatus = LK.getSecretStoreStatus()
            root._busy = false
            root._errorText = ""
            currentField.text = ""
            newField.text = ""
            confirmField.text = ""
            unlockField.text = ""
        }
        root._preserveOnOpen = false
        root._updateButtons()
    }

    onAccepted: {
        root.submit()
        // Ok closes the dialog; reopen after that if work is still in progress or failed.
        Qt.callLater(function() {
            if (root._busy || root._errorText !== "" || unprotectConfirmDialog.visible) {
                root._preserveOnOpen = true
                root.open()
            }
        })
    }

    on_BusyChanged: root._updateButtons()
    onStoreStatusChanged: root._updateButtons()

    Connections {
        target: LK

        function onSecretStoreActionFinished(error, _message) {
            if (!root.visible && !root._busy) {
                return
            }
            root._busy = false
            root.storeStatus = LK.getSecretStoreStatus()
            root._updateButtons()
            if (error !== "") {
                root._errorText = error
                if (!root.visible) {
                    root._preserveOnOpen = true
                    root.open()
                }
            }
            else {
                root.close()
            }
        }
    }

    contentItem: ColumnLayout {
        id: rootColumn
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        spacing: Theme.spacingLoose

        NormalText {
            Layout.fillWidth: true
            visible: root.storeStatus === "NoKey"
            wrapMode: Text.WordWrap
            color: Theme.colorForCriticality("Warning")
            text: "Lightkeeper's secret store encryption key is missing. Restore the encryption key or " +
                  "re-initialize the secrets store here. Re-initializing creates a new key and backs up the old " +
                  "store as secrets.old-YYYY-MM-DDTHH-MM-SS.enc."
        }

        NormalText {
            Layout.fillWidth: true
            visible: root.locked
            wrapMode: Text.WordWrap
            text: "Enter the passphrase to unlock Lightkeeper's secret store. Cancel to continue without it, " +
                  "but hosts that need stored secrets will fail until the store is unlocked."
        }

        NormalText {
            Layout.fillWidth: true
            visible: root.storeStatus === "Uninitialized"
            wrapMode: Text.WordWrap
            text: "Seems that this Lightkeeper Core has not yet been set up.\n" +
                  "Provide a password to set up Lightkeeper's encrypted secret store. " +
                  "Leave it empty to set up with no password protection.\n\n" +
                  "Cancel to continue without initializing Lightkeeper's secret store. " +
                  "You will be prompted again on the next connect."
        }

        NormalText {
            Layout.fillWidth: true
            visible: !root.locked && !root.needsSetup && !root.isProtected
            wrapMode: Text.WordWrap
            text: "Set a passphrase for Lightkeeper's secret store."
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Theme.spacingNormal
            rowSpacing: Theme.spacingNormal
            visible: !root.locked

            Label {
                visible: root.isProtected
                text: "Current passphrase"
            }

            TextField {
                id: currentField
                visible: root.isProtected
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.accept()
            }

            Label {
                text: "New passphrase"
            }

            TextField {
                id: newField
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.accept()
            }

            Label {
                text: "Confirm passphrase"
            }

            TextField {
                id: confirmField
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.accept()
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Theme.spacingNormal
            rowSpacing: Theme.spacingNormal
            visible: root.locked

            Label {
                text: "Passphrase"
            }

            TextField {
                id: unlockField
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.accept()
            }
        }

        SmallText {
            Layout.fillWidth: true
            visible: !root.locked && root._passphraseHint !== ""
            text: root._passphraseHint
            color: Theme.textColorDark
            wrapMode: Text.WordWrap
        }

        Label {
            Layout.fillWidth: true
            visible: root._errorText !== ""
            text: root._errorText
            color: "red"
            wrapMode: Text.WordWrap
        }
    }

    ConfirmationDialog {
        id: unprotectConfirmDialog
        keepHidden: true
        text: "Are you sure you want to remove passphrase protection from Lightkeeper's secret store?"

        onAccepted: {
            root._busy = true
            root._updateButtons()
            LK.setSecretStorePassphrase(currentField.text, "")
        }
    }

    function submit() {
        root._errorText = ""
        if (root.locked) {
            root._busy = true
            root._updateButtons()
            LK.unlockSecretStore(unlockField.text)
            return
        }
        if (newField.text !== confirmField.text) {
            root._errorText = "Passphrases don't match"
            return
        }
        if (newField.text === "" && root.isProtected) {
            unprotectConfirmDialog.open()
            return
        }
        root._busy = true
        root._updateButtons()
        LK.setSecretStorePassphrase(root.isProtected ? currentField.text : "", newField.text)
    }

    function _updateButtons() {
        let okButton = root.standardButton(Dialog.Ok)
        if (okButton) {
            okButton.text = root._okText
            okButton.enabled = !root._busy
        }
        let cancelButton = root.standardButton(Dialog.Cancel)
        if (cancelButton) {
            cancelButton.enabled = !root._busy
        }
    }
}
