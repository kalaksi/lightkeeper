pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts


Item {
    id: root
    width: 600
    height: 400

    property int indentWidth: 20
    property int rowHeight: 28
    property int arrowWidth: 20
    property int nameColumnWidth: 200
    property var columnHeaders: ["Column 1", "Column 2"]
    property var columnWidths: [0.6, 0.4]
    property color headerColor: palette.alternateBase
    property bool useSplitView: false
    property string selectedDirectory: "/"

    property int _maxColumns: 8
    property var _cache: ({})
    property var _expandedDirs: ({})
    property var _currentTreeView: null
    property var _currentDirectoryTreeView: null
    property var _currentFileListView: null

    signal directoryExpanded(string path, bool is_cached)

    onColumnHeadersChanged: {
        if (root.columnHeaders.length > root._maxColumns) {
            console.error("FileBrowser: too many columns, maximum allowed is " + root._maxColumns + ".")
            root.columnHeaders = root.columnHeaders.slice(0, root._maxColumns)
        }
    }

    onColumnWidthsChanged: {
        if (root.columnWidths.length !== root.columnHeaders.length) {
            console.error("FileBrowser: columnWidths count does not match columnHeaders count")
            root.columnWidths = root.columnWidths.slice(0, root.columnHeaders.length)
        }
    }


    Column {
        id: treeViewContainer
        anchors.fill: parent
        visible: !root.useSplitView

        FileBrowserHeader {
            id: header
            width: parent.width
            rowHeight: root.rowHeight
            arrowWidth: root.arrowWidth
            columnHeaders: root.columnHeaders
            headerColor: root.headerColor
            columnWidthProvider: function(column, totalWidth) {
                return root._getColumnWidth(column, totalWidth, false)
            }
        }

        FileBrowserTreeView {
            id: treeView
            width: parent.width
            height: parent.height - header.height
            indentWidth: root.indentWidth
            rowHeight: root.rowHeight
            arrowWidth: root.arrowWidth
            headerColor: root.headerColor
            _cache: root._cache
            _expandedDirs: root._expandedDirs
            _maxColumns: root._maxColumns
            rootPath: "/"
            columnWidthProvider: function(column, totalWidth) {
                return root._getColumnWidth(column, totalWidth, false)
            }

            onDirectoryExpanded: function(path, is_cached) {
                root.directoryExpanded(path, is_cached)
            }

            Component.onCompleted: {
                root._currentTreeView = treeView
            }
        }
    }

    SplitView {
        id: splitViewContainer
        anchors.fill: parent
        orientation: Qt.Horizontal
        visible: root.useSplitView

        FileBrowserTreeView {
            id: directoryTreeView
            indentWidth: root.indentWidth
            rowHeight: root.rowHeight
            arrowWidth: root.arrowWidth
            headerColor: root.headerColor
            _cache: root._cache
            _expandedDirs: root._expandedDirs
            _maxColumns: root._maxColumns
            rootPath: "/"
            hideFiles: true
            columnWidthProvider: function(column, totalWidth) {
                return root._getColumnWidth(column, totalWidth, true)
            }

            SplitView.preferredWidth: parent.width * 0.25
            SplitView.minimumWidth: 100

            onDirectoryExpanded: function(path, is_cached) {
                root.directoryExpanded(path, is_cached)
            }

            onDirectorySelected: function(path) {
                let normalizedPath = root._normalizePath(path)
                root.selectedDirectory = normalizedPath
                if (normalizedPath in root._cache) {
                    fileListView.refreshView()
                }
                else {
                    root.directoryExpanded(normalizedPath, false)
                }
            }

            Component.onCompleted: {
                root._currentDirectoryTreeView = directoryTreeView
            }
        }

        Column {
            SplitView.fillWidth: true

            FileBrowserHeader {
                id: fileHeader
                width: parent.width
                rowHeight: root.rowHeight
                arrowWidth: root.arrowWidth
                columnHeaders: root.columnHeaders
                headerColor: root.headerColor
                columnWidthProvider: function(column, totalWidth) {
                    return root._getColumnWidth(column, totalWidth, false)
                }
            }

            FileBrowserTreeView {
                id: fileListView
                width: parent.width
                height: parent.height - fileHeader.height
                indentWidth: root.indentWidth
                rowHeight: root.rowHeight
                arrowWidth: root.arrowWidth
                headerColor: root.headerColor
                _cache: root._cache
                _expandedDirs: root._expandedDirs
                _maxColumns: root._maxColumns
                rootPath: root.selectedDirectory
                hideDirectories: true
                enableDirectoryNavigation: false
                columnWidthProvider: function(column, totalWidth) {
                    return root._getColumnWidth(column, totalWidth, false)
                }

                onDirectoryExpanded: function(path, is_cached) {
                    root.directoryExpanded(path, is_cached)
                }

                Component.onCompleted: {
                    root._currentFileListView = fileListView
                }
            }
        }
    }

    function openDirectory(dirPath, fileEntries) {
        let normalizedPath = root._normalizePath(dirPath)

        if (fileEntries !== undefined && fileEntries !== null) {
            root._cache[normalizedPath] = fileEntries
        }

        root._expandedDirs[normalizedPath] = true

        if (normalizedPath in root._cache) {
            root.refreshView()
        }
    }

    function refreshView() {
        if (root.useSplitView) {
            if (root._currentDirectoryTreeView) root._currentDirectoryTreeView.refreshView()
            if (root._currentFileListView) root._currentFileListView.refreshView()
        }
        else {
            if (root._currentTreeView) root._currentTreeView.refreshView()
        }
    }

    function toggleDirectory(normalizedPath) {
        if (root.useSplitView) {
            if (root._currentDirectoryTreeView) root._currentDirectoryTreeView.toggleDirectory(normalizedPath)
        }
        else {
            if (root._currentTreeView) root._currentTreeView.toggleDirectory(normalizedPath)
        }
    }

    function clearCache() {
        root._cache = {}
        root._expandedDirs = {}
        root.selectedDirectory = "/"
        root.refreshView()
    }

    function buildEntry(directory, name, fileType, columnData) {
        if (columnData.length !== root.columnHeaders.length) {
            console.error("Column data length does not match column headers length")
            return null
        }
        if (root.columnHeaders.length > root._maxColumns) {
            console.error("FileBrowser: too many columns, maximum allowed is " + root._maxColumns + ".")
            return null
        }

        let fullPath = directory === "/" ? "/" + name : directory + name
        let fullPathStr = String(fullPath)
        if (fileType === "d" && !fullPathStr.endsWith("/")) {
            fullPath = fullPathStr + "/"
        }

        let result = {
            name: name,
            fullPath: fullPath,
            fileType: fileType
        };

        for (let i = 0; i < root._maxColumns; i++) {
            if (i < columnData.length) {
                result["column-" + i] = columnData[i]
            }
            else {
                result["column-" + i] = ""
            }
        }

        return result
    }

    function _normalizePath(path) {
        let pathStr = String(path)
        if (!pathStr.endsWith("/") && pathStr !== "/") {
            path = pathStr + "/"
        }
        return path
    }

    function _getColumnWidth(column, tableViewWidth, hideColumns) {
        if (column === 0) {
            if (hideColumns) {
                return tableViewWidth
            }
            return tableViewWidth * 0.4
        }
        else if (column === 1) {
            return 0
        }
        else if (column === 2) {
            return 0
        }
        else {
            if (hideColumns) {
                return 0
            }

            let columnIndex = column - 3
            if (columnIndex < 0 || columnIndex >= root.columnHeaders.length) {
                return 0
            }

            let totalPercentage = root.columnWidths.reduce((acc, width) => acc + width, 0)
            if (totalPercentage <= 0.0) {
                return 0
            }

            let columnPercentage = root.columnWidths[columnIndex] || 0.0
            return (tableViewWidth * 0.6) * (columnPercentage / totalPercentage)
        }
    }
}
