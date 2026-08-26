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

import Lightkeeper 1.0

import ".."
import Lighthouse.FileBrowser 1.0
import "../Dialog"
import "../Misc"
import "../StyleOverride" as StyleOverride
import "../Text"

Item {
    id: root
    required property string initialPath

    Component {
        id: fileBrowserVerticalScrollBar
        StyleOverride.ScrollBar {
            policy: ScrollBar.AsNeeded
            fadeWhenIdle: false
            anchors.rightMargin: Theme.spacingNormal
        }
    }

    property string hostId: ""
    property int pendingInvocation: 0
    property string pendingPath: ""
    property bool enableShortcuts: false
    property bool _loading: pendingInvocation > 0

    // Invocation ids for which we refresh the file list when they complete (rename, copy, move).
    property var _pendingRefreshInvocationIds: []

    // Copy/Cut/Paste clipboard (file list only).
    property var _fileClipboardPaths: []
    property bool _fileClipboardIsCut: false

    // Paths for which we open the permissions dialog.
    property var _permissionsDialogPaths: []

    // Paths for which we delete files.
    property var _pendingDeletePaths: []

    // Invocation id for a pending mkdir and the path to rename once the refresh completes.
    property int _pendingCreateFolderInvocationId: 0
    property string _pendingRenameAfterRefreshPath: ""

    // Active transfers, ordered newest first.
    ListModel {
        id: transfers
    }

    function _transferIndex(invocationId) {
        for (let i = 0; i < transfers.count; i++) {
            if (transfers.get(i).invocationId === invocationId) {
                return i
            }
        }
        return -1
    }

    function _addTransfer(invocationId) {
        transfers.insert(0, {
            invocationId: invocationId,
            progress: 0,
            statusText: "",
            cancelling: false
        })
    }

    function _updateTransfer(index, progress, statusText, cancelling) {
        transfers.setProperty(index, "progress", progress)
        transfers.setProperty(index, "statusText", statusText)
        transfers.setProperty(index, "cancelling", cancelling)
    }

    Component.onCompleted: {
        fileBrowser.openInitialDirectory(root.initialPath)
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            if (root._pendingCreateFolderInvocationId !== 0
                && root._pendingCreateFolderInvocationId === invocationId) {

                root._pendingCreateFolderInvocationId = 0
                let result = JSON.parse(commandResultJson)
                if (!result.error) {
                    root._pendingRenameAfterRefreshPath =
                        fileBrowser.selectedDirectory + "New folder/"
                    root.refreshCurrentDirectory()
                }
            }
            let transferIndex = root._transferIndex(invocationId)
            if (transferIndex >= 0) {
                let commandResult = JSON.parse(commandResultJson)
                let transfer = transfers.get(transferIndex)
                let statusText = commandResult.progress < 100 && commandResult.message
                    ? commandResult.message
                    : transfer.statusText
                if (commandResult.progress >= 100) {
                    transfers.remove(transferIndex)
                    if (root._pendingRefreshInvocationIds.indexOf(invocationId) >= 0) {
                        root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.filter(id => id !== invocationId)
                        root.refreshCurrentDirectory()
                    }
                }
                else {
                    root._updateTransfer(transferIndex, commandResult.progress,
                        statusText, transfer.cancelling)
                }
            }
            if (root.pendingInvocation === invocationId) {
                let pendingPath = root.pendingPath
                // It's important to to clear these early to avoid race conditions.
                root.pendingPath = ""
                root.pendingInvocation = 0

                let commandResult = JSON.parse(commandResultJson)
                if (commandResult.error) {
                    if (!fileBrowser.navigationError(pendingPath)) {
                        console.error("File browser error:", commandResult.error)
                    }
                    return
                }

                let data = JSON.parse(commandResult.message)
                let browserEntries = data.entries.map(entry => fileBrowser.buildEntry(
                    pendingPath,
                    entry.name,
                    entry.type,
                    [entry.size, entry.time, entry.permissions, entry.owner, entry.group]
                ))

                fileBrowser.openDirectory(pendingPath, browserEntries)

                if (root._pendingRenameAfterRefreshPath !== "") {
                    let renamePath = root._pendingRenameAfterRefreshPath
                    root._pendingRenameAfterRefreshPath = ""
                    Qt.callLater(() => {
                        fileBrowser.selectFilePath(renamePath)
                        fileBrowser.startRenameForSelected()
                    })
                }
            }
            // Non-transfer refreshes (rename, mkdir, delete, chmod, chown). Transfer refreshes
            // are handled above on completion, so don't consume the id while a transfer is
            // still running (partial results would otherwise refresh prematurely).
            if (root._transferIndex(invocationId) < 0 &&
                root._pendingRefreshInvocationIds.indexOf(invocationId) >= 0) {
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
                text: "Back"
                display: AbstractButton.IconOnly
                icon.name: "go-previous"
                icon.height: 24
                icon.width: 24
                padding: 4
                enabled: fileBrowser.canNavigateBack
                onClicked: fileBrowser.navigateBack()

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Back"
            }

            ToolButton {
                flat: false
                text: "Forward"
                display: AbstractButton.IconOnly
                icon.name: "go-next"
                icon.height: 24
                icon.width: 24
                padding: 4
                enabled: fileBrowser.canNavigateForward
                onClicked: fileBrowser.navigateForward()

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Forward"
            }

            ToolSeparator {
                Layout.alignment: Qt.AlignVCenter
            }

            ToolButton {
                flat: false
                icon.source: "qrc:/main/images/button/download"
                text: "Download"
                display: AbstractButton.IconOnly
                onClicked: downloadFolderDialogLoader.active = true
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
                onClicked: uploadFileDialogLoader.active = true
                icon.height: 24
                icon.width: 24
                padding: 4

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Upload files to selected directory"
            }

            ToolButton {
                flat: false
                icon.source: "qrc:/main/images/button/document-open-folder"
                text: "Upload folder"
                display: AbstractButton.IconOnly
                onClicked: uploadFolderDialogLoader.active = true
                icon.height: 24
                icon.width: 24
                padding: 4

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Upload a folder to selected directory"
            }

            StyleOverride.TextField {
                id: pathBar

                Layout.fillWidth: true
                Layout.preferredHeight: 28
                Layout.leftMargin: Theme.spacingTight
                Layout.rightMargin: Theme.spacingNormal
                Layout.alignment: Qt.AlignVCenter
                selectByMouse: true
                readOnly: root._loading
                placeholderText: "Path"
                placeholderTextColor: Theme.textColorDark
                Accessible.name: "Current directory path"

                Component.onCompleted: text = fileBrowser.selectedDirectory

                Connections {
                    target: fileBrowser
                    function onSelectedDirectoryChanged() {
                        pathBar.text = fileBrowser.selectedDirectory
                    }
                }

                onActiveFocusChanged: {
                    if (activeFocus) {
                        selectAll()
                    }
                    else {
                        text = fileBrowser.selectedDirectory
                    }
                }

                Keys.onShortcutOverride: function(event) {
                    if (event.key === Qt.Key_Escape) {
                        event.accepted = true
                    }
                }

                Keys.onEscapePressed: function(event) {
                    text = fileBrowser.selectedDirectory
                    focus = false
                    event.accepted = true
                }

                onAccepted: {
                    let path = text.trim()
                    if (path.length === 0) {
                        path = "/"
                    }
                    fileBrowser.navigateToDirectory(path)
                    focus = false
                }
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

        MenuItem {
            text: "Edit"
            icon.source: "qrc:/main/images/button/story-editor"
            enabled: fileBrowser.selectedFilesOnly.length === 1
            onTriggered: LK.command.openRemoteFileInEditor(root.hostId, fileBrowser.selectedFilesOnly[0])
        }
        MenuItem {
            text: "Create folder"
            icon.source: "qrc:/main/images/button/folder-new"
            onTriggered: root.createFolder()
        }
        MenuItem {
            text: "Download..."
            icon.source: "qrc:/main/images/button/download"
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: downloadFolderDialogLoader.active = true
        }
        MenuItem {
            text: "Rename..."
            icon.source: "qrc:/main/images/button/entry-edit"
            enabled: fileBrowser.hasSingleSelection
            onTriggered: fileBrowser.startRenameForSelected()
        }

        MenuSeparator {
        }

        MenuItem {
            text: "Copy"
            icon.source: "qrc:/main/images/button/copy"
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: root.copySelected()
        }
        MenuItem {
            text: "Cut"
            icon.source: "qrc:/main/images/button/edit-cut"
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: root.cutSelected()
        }
        MenuItem {
            text: "Paste"
            icon.source: "qrc:/main/images/button/edit-paste"
            enabled: root._fileClipboardPaths.length > 0
            onTriggered: root.paste()
        }
        MenuItem {
            text: "Delete"
            icon.source: "qrc:/main/images/button/delete"
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: root.requestDeleteSelectedFiles()
        }

        MenuSeparator {
        }
        
        MenuItem {
            text: "Permissions..."
            icon.source: "qrc:/main/images/button/lock"
            enabled: fileBrowser.selectedFiles.length > 0
            onTriggered: root.openPermissionsDialog()
        }
    }

    FileBrowser {
        id: fileBrowser
        anchors.top: topBar.bottom
        anchors.bottom: transferProgressArea.top
        anchors.left: parent.left
        anchors.right: parent.right
        rootPath: root.initialPath
        directoryTreeRootPath: "/"
        directorySeparator: "/"
        columnHeaders: ["Size", "Modified", "Permissions", "Owner", "Group"]
        columnWidths: [0.35, 0.08]
        headerColor: Theme.backgroundColor
        headerBorderColor: Theme.borderColor
        useSplitView: true
        contextMenu: contextMenu
        directoryIconSource: "qrc:/main/images/button/document-open-folder"
        dimmedPaths: root._fileClipboardIsCut ? root._fileClipboardPaths : []
        verticalScrollBar: fileBrowserVerticalScrollBar
        enableShortcuts: root.enableShortcuts

        onRenamed: function(fullPath, newName) {
            // The file browser re-selects the renamed entry once this refresh completes.
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

    Item {
        id: transferProgressArea
        visible: transfers.count > 0
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        height: visible ? transfers.count * 48 : 0

        Column {
            anchors.fill: parent

            Repeater {
                model: transfers

                FileTransferProgress {
                    width: transferProgressArea.width
                    onStopRequested: root.stopTransfer(invocationId)
                }
            }
        }
    }

    // Using loader because some backends leave some background metadata tasks running even after closing dialog
    Loader {
        id: downloadFolderDialogLoader
        active: false

        sourceComponent: Component {

            FolderDialog {
                title: "Choose download destination"
                currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)

                Component.onCompleted: open()

                onAccepted: {
                    let path = selectedFolder.toString()
                    if (path.indexOf("file://") === 0) {
                        path = path.substring(7)
                    }
                    let localDir = path
                    for (let i = 0; i < fileBrowser.selectedFiles.length; i++) {
                        let remotePath = fileBrowser.selectedFiles[i]
                        let invocationId = LK.command.executePlain(root.hostId, "_internal-filebrowser-download",
                            [remotePath, localDir])
                        root._addTransfer(invocationId)
                    }
                    downloadFolderDialogLoader.active = false
                }

                onRejected: downloadFolderDialogLoader.active = false
            }
        }
    }

    // Using loader because some backends leave some background metadata tasks running even after closing dialog
    Loader {
        id: uploadFileDialogLoader
        active: false

        sourceComponent: Component {

            FileDialog {
                title: "Choose files to upload"
                currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)
                fileMode: FileDialog.OpenFiles

                Component.onCompleted: open()

                onAccepted: {
                    let remoteDir = fileBrowser.selectedDirectory
                    for (let i = 0; i < selectedFiles.length; i++) {
                        let url = selectedFiles[i]
                        let localPath = url.toString()
                        if (localPath.indexOf("file://") === 0) {
                            localPath = localPath.substring(7)
                        }
                        if (localPath.length === 0) {
                            continue
                        }
                        let invocationId = LK.command.executePlain(root.hostId, "_internal-filebrowser-upload",
                            [localPath, remoteDir])
                        root._addTransfer(invocationId)
                        root._pendingRefreshInvocationIds =
                            root._pendingRefreshInvocationIds.concat([invocationId])
                    }
                    uploadFileDialogLoader.active = false
                }

                onRejected: uploadFileDialogLoader.active = false
            }
        }
    }

    // Unfortunately, can't handle both files and folders in the same dialog.
    // Using loader because some backends leave some background metadata tasks running even after closing dialog
    Loader {
        id: uploadFolderDialogLoader
        active: false

        sourceComponent: Component {

            FolderDialog {
                title: "Choose folder to upload"
                currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)

                Component.onCompleted: open()

                onAccepted: {
                    let path = selectedFolder.toString()
                    if (path.indexOf("file://") === 0) {
                        path = path.substring(7)
                    }
                    if (path.length === 0) {
                        uploadFolderDialogLoader.active = false
                        return
                    }
                    let remoteDir = fileBrowser.selectedDirectory
                    let invocationId = LK.command.executePlain(root.hostId, "_internal-filebrowser-upload",
                        [path, remoteDir])
                    root._addTransfer(invocationId)
                    root._pendingRefreshInvocationIds =
                        root._pendingRefreshInvocationIds.concat([invocationId])
                    uploadFolderDialogLoader.active = false
                }

                onRejected: uploadFolderDialogLoader.active = false
            }
        }
    }

    ConfirmationDialog {
        id: deleteConfirmationDialog
        parent: root
        keepHidden: true
        onAccepted: {
            if (root._pendingDeletePaths.length > 0) {
                let id = LK.command.executePlain(root.hostId, "_internal-filebrowser-rm", root._pendingDeletePaths)
                root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([id])
                root._pendingDeletePaths = []
            }
        }
    }

    FilePermissionsDialog {
        id: permissionsDialog
        contextLabel: "Path"
        onPermissionsApplied: function(ownerRwx, groupRwx, othersRwx, newOwner, newGroup) {
            let modeStr = "u=" + ownerRwx + ",g=" + groupRwx + ",o=" + othersRwx
            let initial = root._tripletsFromPermissions(permissionsDialog.permissions)
            let permissionsChanged = root._permissionsDialogPaths.length > 1 ||
                ownerRwx !== initial[0] || groupRwx !== initial[1] || othersRwx !== initial[2]
            let ownershipChanged = newOwner !== permissionsDialog.owner || newGroup !== permissionsDialog.group

            for (let i = 0; i < root._permissionsDialogPaths.length; i++) {
                let path = root._permissionsDialogPaths[i]
                if (permissionsChanged) {
                    let id = LK.command.executePlain(root.hostId,
                        "_internal-filebrowser-chmod", [path, modeStr])
                    root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([id])
                }
                if (ownershipChanged) {
                    let id = LK.command.executePlain(root.hostId,
                        "_internal-filebrowser-chown", [path, newOwner, newGroup])
                    root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([id])
                }
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts && fileBrowser.selectedFiles.length > 0
        sequences: [StandardKey.Copy]
        onActivated: root.copySelected()
    }
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Ctrl+L"
        onActivated: {
            pathBar.forceActiveFocus()
            pathBar.selectAll()
        }
    }
    Shortcut {
        enabled: root.enableShortcuts && fileBrowser.selectedFiles.length > 0
        sequences: [StandardKey.Cut]
        onActivated: root.cutSelected()
    }
    Shortcut {
        enabled: root.enableShortcuts && root._fileClipboardPaths.length > 0
        sequences: [StandardKey.Paste]
        onActivated: root.paste()
    }
    Shortcut {
        enabled: root.enableShortcuts && fileBrowser.hasSingleSelection
        sequence: "F2"
        onActivated: fileBrowser.startRenameForSelected()
    }
    Shortcut {
        enabled: root.enableShortcuts && fileBrowser.selectedFiles.length > 0
        sequences: [StandardKey.Delete]
        onActivated: root.requestDeleteSelectedFiles()
    }

    // Loading animation
    WorkingSprite {
        show: root._loading
    }

    function createFolder() {
        let id = LK.command.executePlain(root.hostId,
            "_internal-filebrowser-mkdir", [fileBrowser.selectedDirectory, "New folder"])
        root._pendingCreateFolderInvocationId = id
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
            let invocationId = LK.command.executePlain(root.hostId,
                "_internal-filebrowser-move", params)
            root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([invocationId])
            root._addTransfer(invocationId)
            root._fileClipboardPaths = []
            root._fileClipboardIsCut = false
        }
        else {
            let invocationId = LK.command.executePlain(root.hostId,
                "_internal-filebrowser-copy", params)
            root._pendingRefreshInvocationIds = root._pendingRefreshInvocationIds.concat([invocationId])
            root._addTransfer(invocationId)
        }
    }

    function stopTransfer(invocationId) {
        let index = root._transferIndex(invocationId)
        if (index >= 0) {
            let transfer = transfers.get(index)
            root._updateTransfer(index, transfer.progress, "Cancelling…", true)
            LK.command.interruptInvocation(invocationId)
        }
    }

    function requestDeleteSelectedFiles() {
        let files = fileBrowser.selectedFiles
        if (files.length === 0) {
            return
        }
        root._pendingDeletePaths = files
        let hasDirectory = files.some(path => path.endsWith("/"))
        deleteConfirmationDialog.text = files.length === 1
            ? "Remove '" + files[0] + "'?"
            : "Remove " + files.length + " selected items?"
        if (hasDirectory) {
            deleteConfirmationDialog.text += "\nDirectories will be removed recursively."
        }

        deleteConfirmationDialog.open()
    }

    function activate() {
        root.enableShortcuts = true
    }

    function deactivate() {
        root.enableShortcuts = false
        fileBrowser.closeFilterBar()
    }

    function refresh() {
        // Clear cache and reopen at the previously selected directory (falling back to the
        // initial path), re-expanding the directory tree segment by segment to reach it.
        let targetPath = fileBrowser.selectedDirectory !== ""
            ? fileBrowser.selectedDirectory
            : fileBrowser.rootPath
        fileBrowser.clearCache()
        fileBrowser.openInitialDirectory(targetPath)
    }

    function refreshCurrentDirectory() {
        root.pendingPath = fileBrowser.selectedDirectory
        root.pendingInvocation = LK.command.listFiles(root.hostId, fileBrowser.selectedDirectory)
    }

    function openPermissionsDialog() {
        let paths = fileBrowser.selectedFiles.slice()
        root._permissionsDialogPaths = paths
        if (paths.length === 1) {
            let path = paths[0]
            permissionsDialog.contextText = path
            permissionsDialog.permissions = fileBrowser.getCellValue(path, 3) ?? ""
            permissionsDialog.owner = fileBrowser.getCellValue(path, 4) ?? ""
            permissionsDialog.group = fileBrowser.getCellValue(path, 5) ?? ""
        }
        else {
            permissionsDialog.contextText = paths.length + " files selected"
            permissionsDialog.permissions = ""
            permissionsDialog.owner = ""
            permissionsDialog.group = ""
        }
        permissionsDialog.open()
    }

    function _tripletsFromPermissions(permStr) {
        if (permStr.length < 9) {
            return ["---", "---", "---"]
        }
        let start = permStr.length === 10 ? 1 : 0
        return [
            permStr.substring(start, start + 3),
            permStr.substring(start + 3, start + 6),
            permStr.substring(start + 6, start + 9)
        ]
    }

    function refreshContent() {
        root.refresh()
    }

    function close() {
    }
}
