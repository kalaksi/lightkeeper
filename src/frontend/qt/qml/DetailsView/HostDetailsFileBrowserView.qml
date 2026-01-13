pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../Text"
import "../StyleOverride"
import "../FileBrowser"

Item {
    id: root
    property string hostId: ""
    property int pendingInvocation: 0
    property string pendingPath: ""
    property string defaultPath: "/"
    property string currentPath: root.defaultPath
    property bool enableShortcuts: false
    property bool _loading: pendingInvocation > 0

    Connections {
        target: LK.command

        function onFileBrowserNavigated(invocationId, directory) {
            root.pendingInvocation = invocationId
            root.pendingPath = directory
        }
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
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

    FileBrowser {
        id: fileBrowser
        anchors.fill: parent
        visible: !root._loading
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
