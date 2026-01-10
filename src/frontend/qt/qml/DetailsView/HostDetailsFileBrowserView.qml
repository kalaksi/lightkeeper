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
    property var content: ({})
    property var directoryCache: ({})
    property string defaultPath: "/"
    property string currentPath: root.defaultPath
    property string _pendingPath: root.defaultPath
    property bool enableShortcuts: false
    property bool _loading: pendingInvocation > 0

    Connections {
        target: LK.command

        function onFileBrowserNavigated(invocationId) {
            // Track the invocation ID - the result will be handled by onCommandResultReceived
            root.pendingInvocation = invocationId
        }
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            if (root.pendingInvocation === invocationId) {
                let commandResult = JSON.parse(commandResultJson)
                root.pendingInvocation = 0

                if (commandResult.error) {
                    console.error("File browser error:", commandResult.error)
                    return
                }

                let data = JSON.parse(commandResult.message)
                let path = root._pendingPath || data.path || root.defaultPath
                
                let parsedContent = root.buildContentFromEntries(data.entries, path)
                root.directoryCache[path] = parsedContent
                
                // Update the content in the tree structure
                root.updateContentTree(path, parsedContent)
                root.updateContent()
            }
        }
    }

    function loadDirectory(path) {
        // Request from server (always refresh to get latest data)
        root._pendingPath = path
        root.pendingInvocation = LK.command.listFiles(root.hostId, path)
    }

    function buildContentFromEntries(entries, currentPath) {
        let result = {}
        for (let entry of entries) {
            if (entry.isDirectory) {
                let dirPath = currentPath === root.defaultPath ? (root.defaultPath + entry.name) : (currentPath + "/" + entry.name)
                if (dirPath in root.directoryCache) {
                    result[entry.name] = root.directoryCache[dirPath]
                } else {
                    // Will be populated when expanded.
                    result[entry.name] = {}
                }
            } else {
                // Column data.
                result[entry.name] = [
                    entry.size,
                    entry.date,
                    entry.time,
                    entry.permissions,
                    entry.owner,
                    entry.group
                ]
            }
        }
        return result
    }

    function updateContent() {
        // Build content tree from cache, starting from current path
        if (root.currentPath in root.directoryCache) {
            root.content = root.directoryCache[root.currentPath]
        } else {
            root.content = {}
        }
    }

    function updateContentTree(path, content) {
        // Update the content tree structure to reflect the new directory data
        // This ensures parent directories show the updated child data
        if (path === root.defaultPath || path === root.currentPath) {
            root.content = content
        } else {
            // Update nested structure - find parent and update
            root.updateNestedContent(path, content)
        }
    }

    function updateNestedContent(path, content) {
        // Recursively update nested content structure
        let pathParts = path.split("/").filter(p => p !== "")
        if (pathParts.length === 0) {
            root.content = content
            return
        }

        let current = root.content
        for (let i = 0; i < pathParts.length - 1; i++) {
            if (current[pathParts[i]] && typeof current[pathParts[i]] === 'object' && !Array.isArray(current[pathParts[i]])) {
                current = current[pathParts[i]]
            } else {
                return // Path doesn't exist in structure
            }
        }
        
        // Update the final directory
        let dirName = pathParts[pathParts.length - 1]
        if (current[dirName] && typeof current[dirName] === 'object' && !Array.isArray(current[dirName])) {
            // Merge new content into existing
            for (let key in content) {
                current[dirName][key] = content[key]
            }
        }
    }

    function onDirectoryExpanded(path) {
        // Load directory contents when expanded
        root.loadDirectory(path)
    }

    Rectangle {
        color: Theme.backgroundColor
        anchors.fill: parent
    }

    FileBrowser {
        id: fileBrowser
        anchors.fill: parent
        visible: !root._loading
        content: root.content
        columnHeaders: ["Size", "Date", "Time", "Permissions", "Owner", "Group"]
        columnWidths: [80, 100, 80, 100, 80, 80]
        headerColor: Theme.backgroundColorDark
        dirColor: Theme.backgroundColorLight
        fileColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        hoverColor: Theme.highlightColorLight
        onDirectoryExpanded: root.onDirectoryExpanded
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
        if (Object.keys(root.content).length === 0) {
            root.loadDirectory(root.defaultPath)
        }
    }

    function deactivate() {
        root.enableShortcuts = false
    }

    function refresh() {
        // Clear cache and reload current directory
        root.directoryCache = {}
        root.loadDirectory(root.currentPath)
    }

    function close() {
        // Cleanup if needed
    }

    Component.onCompleted: {
        // Only load if we don't already have a pending invocation (i.e., not created with an existing invocation)
        if (root.pendingInvocation === 0) {
            root.loadDirectory(root.defaultPath)
        }
    }
}
