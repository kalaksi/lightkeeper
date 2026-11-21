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

    readonly property bool locked: root.storeStatus === "Locked"
    readonly property bool needsSetup: root.storeStatus === "Uninitialized" || root.storeStatus === "NoKey"
    readonly property bool isProtected: root.storeStatus === "UnlockedWithPassphrase"
    readonly property string _passwordHint: root.needsSetup ? "Leave empty for no password protection."
        : root.isProtected ? "Leave empty to remove protection."
        : ""
    readonly property string _okText: root.locked ? "Unlock"
        : root.storeStatus === "NoKey" ? "Re-initialize"
        : root.storeStatus === "Uninitialized" ? "Set up"
        : root.isProtected ? "Change password"
        : "Set password"

    title: root.locked ? "Unlock Lightkeeper's secret store"
        : root.storeStatus === "NoKey" ? "Re-initialize Lightkeeper's secret store"
        : root.storeStatus === "Uninitialized" ? "Set up Lightkeeper Core"
        : root.isProtected ? "Change password"
        : "Set password"
    modal: true
    implicitWidth: 520
    implicitHeight: rootColumn.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    // ApplyRole applies without closing (unlike Ok / AcceptRole). Native DialogButtonBox footer.
    standardButtons: Dialog.Apply | Dialog.Cancel
    showCloseButton: !root._busy
    closePolicy: root._busy ? Popup.NoAutoClose : Popup.CloseOnEscape

    onOpened: {
        root.storeStatus = LK.getSecretStoreStatus()
        root._updateButtons()
    }

    onClosed: root._clearForm()

    onApplied: root.submit()

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
        anchors.bottomMargin: Theme.marginDialogBottom
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
            text: "Enter the password to unlock Lightkeeper's secret store. Cancel to continue without it, " +
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
            text: "Set a password to protect Lightkeeper's encrypted secret store."
        }

        NormalText {
            Layout.fillWidth: true
            visible: root.isProtected
            wrapMode: Text.WordWrap
            text: "Enter the current password and a new one for Lightkeeper's encrypted secret store.\n" +
                  "Leave the new password empty to remove password protection."
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Theme.spacingNormal
            rowSpacing: Theme.spacingNormal
            visible: !root.locked

            Label {
                visible: root.isProtected
                text: "Current password"
            }

            TextField {
                id: currentField
                visible: root.isProtected
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.submit()
            }

            Label {
                text: "New password"
            }

            TextField {
                id: newField
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.submit()
            }

            Label {
                text: "Confirm password"
            }

            TextField {
                id: confirmField
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.submit()
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Theme.spacingNormal
            rowSpacing: Theme.spacingNormal
            visible: root.locked

            Label {
                text: "Password"
            }

            TextField {
                id: unlockField
                echoMode: TextInput.Password
                enabled: !root._busy
                Layout.fillWidth: true
                onAccepted: root.submit()
            }
        }

        SmallText {
            Layout.fillWidth: true
            visible: !root.locked && root._passwordHint !== ""
            text: root._passwordHint
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

        // Reserved slot so showing the sprite does not resize the dialog.
        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: 36
            opacity: root._busy ? 1 : 0

            WorkingSprite {
            }
        }
    }

    function _clearForm() {
        root._busy = false
        root._errorText = ""
        currentField.text = ""
        newField.text = ""
        confirmField.text = ""
        unlockField.text = ""
    }

    function submit() {
        if (root._busy) {
            return
        }
        root._errorText = ""
        if (root.locked) {
            root._busy = true
            root._updateButtons()
            LK.unlockSecretStore(unlockField.text)
            return
        }
        if (newField.text !== confirmField.text) {
            root._errorText = "Passwords don't match"
            return
        }
        root._busy = true
        root._updateButtons()
        LK.setSecretStorePassphrase(root.isProtected ? currentField.text : "", newField.text)
    }

    function _updateButtons() {
        let applyButton = root.standardButton(Dialog.Apply)
        if (applyButton) {
            applyButton.text = root._okText
            applyButton.enabled = !root._busy
        }
        let cancelButton = root.standardButton(Dialog.Cancel)
        if (cancelButton) {
            cancelButton.enabled = !root._busy
        }
    }
}
