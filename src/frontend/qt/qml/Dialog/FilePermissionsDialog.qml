/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Lightkeeper 1.0

import Lighthouse.FilePermissionsDialog 1.0

LightkeeperDialog {
    id: root

    property alias contextLabel: permissionsContent.contextLabel
    property alias contextText: permissionsContent.contextText
    property alias permissions: permissionsContent.permissions
    property alias owner: permissionsContent.owner
    property alias group: permissionsContent.group
    property alias warningText: permissionsContent.warningText
    property alias showOctal: permissionsContent.showOctal
    property alias showSpecialBits: permissionsContent.showSpecialBits

    signal permissionsApplied(string mode, string changedOwner, string changedGroup)

    title: "Permissions and ownership"
    modal: true
    leftPadding: Theme.marginDialog + Theme.spacingLoose
    rightPadding: Theme.marginDialog
    implicitWidth: Math.max(400, permissionsContent.implicitWidth + leftPadding + rightPadding)
    // implicitHeight: permissionsContent.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    onOpened: {
        permissionsContent._updateFromProps()
        Qt.callLater(_updateOkButton)
    }

    Connections {
        target: permissionsContent
        function onCanAcceptChanged() {
            root._updateOkButton()
        }
    }

    contentItem: FilePermissionsDialogContent {
        id: permissionsContent
        contentMargin: 0
        sectionSpacing: Theme.spacingLoose
        rowSpacing: 4
        marginTop: Theme.marginDialogTop
        marginBottom: root.warningText.length > 0 ? Theme.spacingLoose : Theme.marginDialogBottom
        comboMinWidth: 200
        fontSize: Theme.fontSize
        optionSpacing: Theme.spacingNormal * 2
        showSpecialBits: true
        tooltipDelay: Theme.tooltipDelay
        warningTextColor: Theme.textColorDark
    }

    onAccepted: {
        if (permissionsContent.canAccept && permissionsContent.hasChanges) {
            root.permissionsApplied(permissionsContent.resultMode, permissionsContent.changedOwner, permissionsContent.changedGroup)
        }
    }

    function _updateOkButton() {
        if (root.visible && root.standardButton(Dialog.Ok))
            root.standardButton(Dialog.Ok).enabled = permissionsContent.canAccept
    }
}
