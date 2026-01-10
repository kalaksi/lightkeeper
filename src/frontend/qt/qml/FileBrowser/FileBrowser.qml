pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import Qt.labs.qmlmodels

Item {
    id: root
    width: 600
    height: 400

    property int indentWidth: 20
    property int rowHeight: 28
    property int arrowWidth: 20
    property int nameColumnWidth: 200
    property var columnHeaders: ["Column 1", "Column 2"]
    property var columnWidths: [200, 200]
    property color headerColor: palette.alternateBase

    // Cache structure: cache[path] = fileEntries (raw entries from backend)
    property var _cache: ({})
    // Track expanded state: expandedPaths[path] = true/false
    property var _expandedDirs: ({})
    property int _maxColumns: 8

    // Signal for directory expansion with is_cached parameter
    signal directoryExpanded(string path, bool is_cached)

    onColumnHeadersChanged: {
        if (root.columnHeaders.length > root._maxColumns) {
            console.error("FileBrowser: too many columns, maximum allowed is " + root._maxColumns + ".")
            root.columnHeaders = root.columnHeaders.slice(0, root._maxColumns)
        }
    }

    // Calculate total width of data columns
    readonly property int dataColumnsWidth: {
        var total = 0
        for (var i = 0; i < columnWidths.length; i++)
            total += columnWidths[i]
        return total
    }


    Column {
        anchors.fill: parent

        // Header
        Rectangle {
            width: parent.width
            height: root.rowHeight
            color: root.headerColor

            Row {
                anchors.fill: parent
                anchors.leftMargin: root.arrowWidth

                Label {
                    width: root.nameColumnWidth
                    text: "Name"
                    font: palette.buttonText
                    verticalAlignment: Text.AlignVCenter
                    elide: Text.ElideRight
                }

                Repeater {
                    model: root.columnHeaders.length

                    Label {
                        required property int index

                        width: root._getColumnWidth(index + 3)
                        text: root.columnHeaders[index]
                        font: palette.buttonText
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideRight
                    }
                }
            }
        }

        TableView {
            id: tableView
            width: parent.width
            height: parent.height - root.rowHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            onWidthChanged: forceLayout()
            // Somehow shows warning if not using arrow function here.
            columnWidthProvider: (column) => root._getColumnWidth(column)
            rowHeightProvider: function(row) {
                return root.rowHeight
            }

            model: TableModel {
                id: tableModel

                TableModelColumn { display: "name" }
                TableModelColumn { display: "fullPath" }
                TableModelColumn { display: "fileType" }
                // Count matches root._maxColumns, dynamically added.
                TableModelColumn { display: "column-0" }
                TableModelColumn { display: "column-1" }
                TableModelColumn { display: "column-2" }
                TableModelColumn { display: "column-3" }
                TableModelColumn { display: "column-4" }
                TableModelColumn { display: "column-5" }
                TableModelColumn { display: "column-6" }
                TableModelColumn { display: "column-7" }
            }

            ScrollBar.vertical: ScrollBar {
                active: true
            }

            delegate: TableViewDelegate {
                id: viewDelegate

                property string fullPath: {
                    if (viewDelegate.row >= 0 && viewDelegate.row < tableModel.rowCount && tableModel.rows) {
                        let rowData = tableModel.rows[viewDelegate.row]
                        return rowData && rowData.fullPath ? String(rowData.fullPath) : ""
                    }
                    return ""
                }
                
                property string name: {
                    if (viewDelegate.row >= 0 && viewDelegate.row < tableModel.rowCount && tableModel.rows) {
                        let rowData = tableModel.rows[viewDelegate.row]
                        return rowData && rowData.name ? String(rowData.name) : ""
                    }
                    return ""
                }
                property string fileType: {
                    if (viewDelegate.row >= 0 && viewDelegate.row < tableModel.rowCount && tableModel.rows) {
                        let rowData = tableModel.rows[viewDelegate.row]
                        return rowData && rowData.fileType ? String(rowData.fileType) : ""
                    }
                    return ""
                }

                onClicked: {
                    if (viewDelegate.fileType === "d") {
                        root.toggleDirectory(viewDelegate.fullPath)
                    }
                }

                contentItem: Item {
                    Row {
                        id: nameColumn
                        visible: viewDelegate.column === 0
                        height: parent.height
                        spacing: 4

                        // Arrow/indent area
                        Item {
                            id: arrowIndentArea
                            // TODO: is naive split accurate enough?
                            property int depth: viewDelegate.fullPath.split("/").length - 1

                            width: root.arrowWidth + (arrowIndentArea.depth * root.indentWidth)
                            height: parent.height

                            Text {
                                anchors.left: parent.left
                                anchors.leftMargin: (arrowIndentArea.depth * root.indentWidth)
                                anchors.verticalCenter: parent.verticalCenter
                                width: root.arrowWidth
                                visible: viewDelegate.fileType === "d"
                                text: root._expandedDirs[viewDelegate.fullPath] === true ? "▼" : "▶"
                                font: viewDelegate.font
                                color: viewDelegate.highlighted ? viewDelegate.palette.highlightedText : viewDelegate.palette.buttonText
                                verticalAlignment: Text.AlignVCenter
                            }
                        }

                        Label {
                            width: root.nameColumnWidth
                            text: viewDelegate.model.display
                            elide: Text.ElideRight
                            color: viewDelegate.highlighted ? viewDelegate.palette.highlightedText : viewDelegate.palette.buttonText
                            verticalAlignment: Text.AlignVCenter
                        }
                    }

                    Label {
                        visible: viewDelegate.column > 2
                        height: parent.height
                        text: viewDelegate.model.display
                        elide: Text.ElideRight
                        color: viewDelegate.highlighted ? viewDelegate.palette.highlightedText : viewDelegate.palette.buttonText
                    }
                }
            }
        }
    }

    // Sort entries by name, directories first.
    function _sortEntries(entries) {
        return [...entries].sort(function(a, b) {
            if (a.fileType === "d" && b.fileType !== "d") {
                return -1
            }
            else if (a.fileType !== "d" && b.fileType === "d") {
                return 1
            }
            return a.name.localeCompare(b.name)
        })
    }

    // Build table model entries from cache. Path has to exist in cache.
    function _buildFlatList(normalizedDirPath, depth = 0) {
        let result = []
        let entries = root._cache[normalizedDirPath]
        let sortedEntries = root._sortEntries(entries)

        for (let entry of sortedEntries) {
            result.push(entry)

            // If directory is expanded, recursively add its children
            let isExpanded = root._expandedDirs[entry.fullPath] === true
            if (entry.fileType === "d" && isExpanded) {
                let children = root._buildFlatList(entry.fullPath, depth + 1)
                result.push(...children)
            }
        }

        return result
    }

    function _getColumnWidth(column) {
        if (column === 0) {
            return root.arrowWidth + root.nameColumnWidth
        }
        else if (column === 1) {
            return 0
        }
        else if (column === 2) {
            return 0
        }
        else {
            return root.columnWidths[column - 2] || 0
        }
    }

    function _normalizePath(path) {
        if (!path.endsWith("/") && path !== "/") {
            path = path + "/"
        }
        return path
    }

    function openDirectory(dirPath, fileEntries) {
        let normalizedPath = root._normalizePath(dirPath)

        if (fileEntries !== undefined && fileEntries !== null) {
            // TODO: save dir path?
            root._cache[normalizedPath] = fileEntries
        }

        root._expandedDirs[normalizedPath] = true

        if (normalizedPath in root._cache) {
            root.refreshView()
        }
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
        if (fileType === "d" && !fullPath.endsWith("/")) {
            fullPath = fullPath + "/"
        }

        // Create row object with properties matching column headers for TableModel
        let result = {
            name: name,
            fullPath: fullPath,
            fileType: fileType
        };

        // Add column values as direct properties, pad to _maxColumns
        for (let i = 0; i < root._maxColumns; i++) {
            if (i < columnData.length) {
                result["column-" + i] = columnData[i]
            }
            else {
                // Pad with empty string for missing columns
                result["column-" + i] = ""
            }
        }

        return result
    }

    function refreshView() {
        tableModel.clear()

        let flatList = root._buildFlatList("/")
        for (let row of flatList) {
            tableModel.appendRow(row)
        }
    }

    // Toggle directory expansion
    function toggleDirectory(normalizedPath) {
        let isCurrentlyExpanded = root._expandedDirs[normalizedPath] === true
        let isCached = root._cache[normalizedPath] !== undefined

        if (isCurrentlyExpanded) {
            root._expandedDirs[normalizedPath] = false
        }
        else {
            root._expandedDirs[normalizedPath] = true
            root.directoryExpanded(normalizedPath, isCached)
        }

        if (isCached) {
            root.refreshView()
        }
    }

    // Clear the cache
    function clearCache() {
        root._cache = {}
        root._expandedDirs = {}
        tableModel.clear()
    }
}
