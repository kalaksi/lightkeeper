/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import QtCore

import Theme

import ".."
import "../FileBrowser"
import "../Misc"
import "../Text"

Item {
    id: root
    property string hostId: ""
    property int pendingInvocation: 0
    property string pendingPath: ""
    property string defaultPath: "/"
    property string currentPath: root.defaultPath
    property bool enableShortcuts: false
    property bool _loading: pendingInvocation > 0
    property int _downloadProgressPercent: 0
    property bool _hasActiveDownload: false
    property var _downloadInvocations: ({})

    Connections {
        target: LK.command

        function onFileBrowserNavigated(invocationId, directory) {
            root.pendingInvocation = invocationId
            root.pendingPath = directory
        }
    }

    function _minDownloadProgress() {
        let invs = root._downloadInvocations
        let keys = Object.keys(invs)
        if (keys.length === 0) {
            return 100
        }
        let minP = 100
        for (let i = 0; i < keys.length; i++) {
            let p = invs[keys[i]]
            if (p < minP) {
                minP = p
            }
        }
        return minP
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            if (invocationId in root._downloadInvocations) {
                let commandResult = JSON.parse(commandResultJson)
                root._downloadInvocations[invocationId] = commandResult.progress
                if (commandResult.progress >= 100) {
                    delete root._downloadInvocations[invocationId]
                }
                root._downloadProgressPercent = root._minDownloadProgress()
                if (Object.keys(root._downloadInvocations).length === 0) {
                    root._hasActiveDownload = false
                }
            }
            if (root.pendingInvocation === invocationId) {
                let dirPath = root.pendingPath

                root.pendingPath = ""
                root.pendingInvocation = 0

                let commandResult = JSON.parse(commandResultJson)
                if (commandResult.error) {
                    console.error("File browser error:", commandResult.error)
                    return
                }

                let data = JSON.parse(commandResult.message)
                let browserEntries = data.entries.map(entry => fileBrowser.buildEntry(
                    dirPath,
                    entry.name,
                    entry.type,
                    [entry.size, entry.time, entry.permissions, entry.owner, entry.group]
                ))

                fileBrowser.openDirectory(dirPath, browserEntries)
            }
        }
    }

    Rectangle {
        color: Theme.backgroundColor
        anchors.fill: parent
    }

    ToolBar {
        id: topBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 36

        background: BorderRectangle {
            backgroundColor: Theme.backgroundColor
            borderColor: Theme.borderColor
            borderBottom: 1
        }

        RowLayout {
            width: parent.width
            height: parent.height
            anchors.top: parent.top
            spacing: Theme.spacingNormal

            ToolButton {
                icon.source: "qrc:/main/images/button/download"
                text: "Download"
                display: AbstractButton.IconOnly
                onClicked: downloadFolderDialog.open()
                enabled: fileBrowser.selectedFiles.length > 0
                icon.height: 24
                icon.width: 24
                padding: 4

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Download selected files"
            }

            Item {
                Layout.fillWidth: true
            }
        }
    }

    FileBrowser {
        id: fileBrowser
        anchors.top: topBar.bottom
        anchors.bottom: downloadProgressBar.top
        anchors.left: parent.left
        anchors.right: parent.right
        columnHeaders: ["Size", "Modified", "Permissions", "Owner", "Group"]
        columnWidths: [0.1, 0.3, 0.2, 0.2, 0.2]
        useSplitView: true

        onDirectoryExpanded: function(path, is_cached) {
            if (!is_cached) {
                root.pendingPath = path
                root.pendingInvocation = LK.command.listFiles(root.hostId, path)
            }
            else {
                fileBrowser.openDirectory(path)
            }
        }
    }

    Rectangle {
        id: downloadProgressBar
        visible: root._hasActiveDownload
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: visible ? 28 : 0
        color: Theme.backgroundColor
        border.width: 1
        border.color: Theme.borderColor

        RowLayout {
            anchors.fill: parent
            anchors.margins: 4
            spacing: Theme.spacingNormal

            ProgressBar {
                id: progressBar
                Layout.fillWidth: true
                Layout.fillHeight: false
                Layout.preferredHeight: 18
                Layout.alignment: Qt.AlignVCenter
                value: root._downloadProgressPercent / 100.0

                contentItem: Rectangle {
                    implicitHeight: progressBar.height
                    implicitWidth: progressBar.width
                    color: "#202020"
                    radius: 4

                    Rectangle {
                        height: parent.height
                        width: progressBar.visualPosition * parent.width
                        color: palette.highlight
                        radius: parent.radius
                    }
                }
            }

            NormalText {
                id: label
                lineHeight: 0.9
                text: root._downloadProgressPercent + " %"
                Layout.alignment: Qt.AlignVCenter
            }
        }
    }

    FolderDialog {
        id: downloadFolderDialog
        title: "Choose download destination"
        currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)

        onAccepted: {
            let path = selectedFolder.toString()
            if (path.indexOf("file://") === 0) {
                path = path.substring(7)
            }
            let localDir = path
            let remoteUser = LK.config.getSshUsername(root.hostId)
            root._hasActiveDownload = true
            root._downloadProgressPercent = 0
            for (let i = 0; i < fileBrowser.selectedFiles.length; i++) {
                let remotePath = fileBrowser.selectedFiles[i]
                let invId = LK.command.executePlain(root.hostId, "_internal-filebrowser-download",
                    [remotePath, localDir, remoteUser])
                let invs = root._downloadInvocations
                invs[invId] = 0
                root._downloadInvocations = invs
            }
        }
    }

    // Loading animation
    Item {
        visible: root._loading
        anchors.fill: parent

        WorkingSprite {
        }
    }

    function activate() {
        root.enableShortcuts = true
    }

    function deactivate() {
        root.enableShortcuts = false
    }

    function refresh() {
        // Clear cache and reload current directory.
        fileBrowser.clearCache()
        root.pendingPath = root.currentPath
        root.pendingInvocation = LK.command.listFiles(root.hostId, root.currentPath)
    }

    function close() {
    }
}
