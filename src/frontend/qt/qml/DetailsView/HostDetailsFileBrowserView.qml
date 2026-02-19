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
import Lighthouse.FileBrowser 1.0
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

    // Related to downloads.
    property int _transferProgressPercent: 0
    property bool _hasActiveTransfer: false
    property var _transferInvocations: ({})

    // Invocation ids for which we refresh the file list when they complete (rename, copy, move).
    property var _pendingRefreshInvocationIds: []

    // Copy/Cut/Paste clipboard (file list only).
    property var _fileClipboardPaths: []
    property bool _fileClipboardIsCut: false

    Connections {
        target: LK.command

        function onFileBrowserNavigated(invocationId, directory) {
            root.pendingInvocation = invocationId
            root.pendingPath = directory
        }
    }

    function _minTransferProgress() {
        let invs = root._transferInvocations
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
            if (invocationId in root._transferInvocations) {
                let commandResult = JSON.parse(commandResultJson)
                root._transferInvocations[invocationId] = commandResult.progress
                if (commandResult.progress >= 100) {
                    delete root._transferInvocations[invocationId]
                    if (root._pendingRefreshInvocationIds.indexOf(invocationId) >= 0) {
                        root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.filter(id => id !== invocationId)
                        root.refreshCurrentDirectory()
                    }
                }
                root._transferProgressPercent = root._minTransferProgress()
                if (Object.keys(root._transferInvocations).length === 0) {
                    root._hasActiveTransfer = false
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
            if (root._pendingRefreshInvocationIds.indexOf(invocationId) >= 0) {
                root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.filter(id => id !== invocationId)
                root.refreshCurrentDirectory()
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
            spacing: Theme.spacingTight

            ToolButton {
                flat: false
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

            ToolButton {
                flat: false
                icon.source: "qrc:/main/images/button/upload"
                text: "Upload"
                display: AbstractButton.IconOnly
                onClicked: uploadFileDialog.open()
                icon.height: 24
                icon.width: 24
                padding: 4

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Upload files to selected directory"
            }

            Item {
                Layout.fillWidth: true
            }

            Row {
                visible: fileBrowser.useSplitView
                spacing: Theme.spacingLoose

                Layout.alignment: Qt.AlignVCenter

                Label {
                    text: "Show directories"
                }

                CheckBox {
                    checked: !fileBrowser.hideDirectories
                    onToggled: fileBrowser.hideDirectories = !checked
                }
            }
        }
    }

    Menu {
        id: contextMenu

        MenuSeparator {
        }
        MenuItem {
            text: "Copy"
            icon.source: "qrc:/main/images/button/copy"
            icon.width: 22
            icon.height: 22
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: root.copySelected()
        }
        MenuItem {
            text: "Cut"
            icon.source: "qrc:/main/images/button/edit-cut"
            icon.width: 22
            icon.height: 22
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: root.cutSelected()
        }
        MenuItem {
            text: "Paste"
            icon.source: "qrc:/main/images/button/edit-paste"
            icon.width: 22
            icon.height: 22
            enabled: root._fileClipboardPaths.length > 0
            onTriggered: root.paste()
        }
        MenuSeparator {
        }
        MenuItem {
            text: "Download..."
            icon.source: "qrc:/main/images/button/download"
            icon.width: 22
            icon.height: 22
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: downloadFolderDialog.open()
        }
        MenuItem {
            text: "Edit"
            icon.source: "qrc:/main/images/button/story-editor"
            icon.width: 22
            icon.height: 22
            enabled: fileBrowser.selectedFiles.length === 1
            onTriggered: LK.command.openRemoteFileInEditor(root.hostId, fileBrowser.selectedFiles[0])
        }
        MenuItem {
            text: "Rename..."
            icon.source: "qrc:/main/images/button/entry-edit"
            icon.width: 22
            icon.height: 22
            enabled: fileBrowser.hasSingleSelection
            onTriggered: fileBrowser.startRenameForSelected()
        }
    }

    FileBrowser {
        id: fileBrowser
        anchors.top: topBar.bottom
        anchors.bottom: transferProgressBar.top
        anchors.left: parent.left
        anchors.right: parent.right
        columnHeaders: ["Size", "Modified", "Permissions", "Owner", "Group"]
        columnWidths: [0.35, 0.08]
        headerColor: Theme.backgroundColor
        headerBorderColor: Theme.borderColor
        useSplitView: true
        contextMenu: contextMenu
        directoryIconSource: "qrc:/main/images/button/document-open-folder"

        onRenamed: function(fullPath, newName) {
            let id = LK.command.executePlain(root.hostId,
                "_internal-filebrowser-rename", [fullPath, newName])
            root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([id])
        }

        onDirectoryExpanded: function(path, isCached) {
            if (!isCached) {
                root.pendingPath = path
                root.pendingInvocation = LK.command.listFiles(root.hostId, path)
            }
            else {
                fileBrowser.openDirectory(path)
            }
        }
    }

    Rectangle {
        id: transferProgressBar
        visible: root._hasActiveTransfer
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
                value: root._transferProgressPercent / 100.0

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
                text: root._transferProgressPercent + " %"
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
            root._hasActiveTransfer = true
            root._transferProgressPercent = 0
            for (let i = 0; i < fileBrowser.selectedFiles.length; i++) {
                let remotePath = fileBrowser.selectedFiles[i]
                let invocationId = LK.command.executePlain(root.hostId, "_internal-filebrowser-download",
                    [remotePath, localDir, remoteUser])
                root._transferInvocations[invocationId] = 0
            }
        }
    }

    FileDialog {
        id: uploadFileDialog
        title: "Choose files to upload"
        currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)
        fileMode: FileDialog.OpenFiles

        onAccepted: {
            let remoteDir = fileBrowser.selectedDirectory
            let remoteUser = LK.config.getSshUsername(root.hostId)
            root._hasActiveTransfer = true
            root._transferProgressPercent = 0
            for (let i = 0; i < selectedFiles.length; i++) {
                let url = selectedFiles[i]
                let localPath = url.toString()
                if (localPath.indexOf("file://") === 0) {
                    localPath = localPath.substring(7)
                }
                if (localPath.length === 0) {
                    continue
                }
                let invId = LK.command.executePlain(root.hostId, "_internal-filebrowser-upload",
                    [localPath, remoteDir, remoteUser])
                let invs = root._transferInvocations
                invs[invId] = 0
                root._transferInvocations = invs
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts && fileBrowser.selectedFiles.length > 0
        sequence: StandardKey.Copy
        onActivated: root.copySelected()
    }
    Shortcut {
        enabled: root.enableShortcuts && fileBrowser.selectedFiles.length > 0
        sequence: StandardKey.Cut
        onActivated: root.cutSelected()
    }
    Shortcut {
        enabled: root.enableShortcuts && root._fileClipboardPaths.length > 0
        sequence: StandardKey.Paste
        onActivated: root.paste()
    }

    // Loading animation
    WorkingSprite {
        show: root._loading
    }

    function copySelected() {
        root._fileClipboardPaths = fileBrowser.selectedFiles.slice()
        root._fileClipboardIsCut = false
    }

    function cutSelected() {
        root._fileClipboardPaths = fileBrowser.selectedFiles.slice()
        root._fileClipboardIsCut = true
    }

    function paste() {
        if (root._fileClipboardPaths.length === 0) {
            return
        }
        let dest = fileBrowser.selectedDirectory
        let params = [dest].concat(root._fileClipboardPaths)
        if (root._fileClipboardIsCut) {
            let id = LK.command.executePlain(root.hostId,
                "_internal-filebrowser-move", params)
            root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([id])
            root._fileClipboardPaths = []
            root._fileClipboardIsCut = false
        }
        else {
            let invocationId = LK.command.executePlain(root.hostId,
                "_internal-filebrowser-copy", params)
            root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([invocationId])
            root._hasActiveTransfer = true
            root._transferInvocations[invocationId] = 0
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

    function refreshCurrentDirectory() {
        root.pendingPath = fileBrowser.selectedDirectory
        root.pendingInvocation = LK.command.listFiles(root.hostId, fileBrowser.selectedDirectory)
    }

    function close() {
    }
}
