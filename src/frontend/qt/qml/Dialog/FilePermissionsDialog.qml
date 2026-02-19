/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Theme

import Lighthouse.FilePermissionsDialog 1.0

LightkeeperDialog {
    id: root

    property alias contextLabel: permissionsContent.contextLabel
    property alias contextText: permissionsContent.contextText
    property alias permissions: permissionsContent.permissions
    property alias owner: permissionsContent.owner
    property alias group: permissionsContent.group

    signal applied(string ownerRwx, string groupRwx, string othersRwx, string newOwner, string newGroup)

    title: "Permissions and ownership"
    modal: true
    implicitWidth: permissionsContent.implicitWidth + Theme.marginDialog * 2
    // implicitHeight: permissionsContent.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    onOpened: {
        permissionsContent._updateFromProps()
        Qt.callLater(_updateOkButton)
    }

    Connections {
        target: permissionsContent
        function onCanAcceptChanged() {
            _updateOkButton()
        }
    }

    contentItem: FilePermissionsDialogContent {
        id: permissionsContent
        contentMargin: Theme.marginDialog
        sectionSpacing: Theme.spacingLoose
        rowSpacing: 4
        marginTop: Theme.marginDialogTop
        marginBottom: Theme.marginDialogBottom
        comboMinWidth: 200
        fontSize: Theme.fontSize
    }

    onAccepted: {
        if (permissionsContent.canAccept)
            root.applied(permissionsContent.resultOwnerRwx, permissionsContent.resultGroupRwx,
                permissionsContent.resultOthersRwx, permissionsContent.resultOwner,
                permissionsContent.resultGroup)
    }

    function _updateOkButton() {
        if (root.visible && root.standardButton(Dialog.Ok))
            root.standardButton(Dialog.Ok).enabled = permissionsContent.canAccept
    }
}
