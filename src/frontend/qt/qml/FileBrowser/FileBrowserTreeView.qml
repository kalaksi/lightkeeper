pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import Qt.labs.qmlmodels

Item {
    id: root

    required property var columnWidthProvider

    property int indentWidth: 20
    property int rowHeight: 28
    property int arrowWidth: 20
    property color headerColor: palette.alternateBase
    property string rootPath: "/"
    property bool hideFiles: false
    property bool hideDirectories: false
    property bool enableDirectoryNavigation: true

    property var _cache: ({})
    property var _expandedDirs: ({})
    property int _maxColumns: 8


    signal directoryExpanded(string path, bool is_cached)
    signal directorySelected(string path)

    onRootPathChanged: refreshView()

    TableView {
        id: tableView
        anchors.fill: parent
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        onWidthChanged: forceLayout()
        columnWidthProvider: (column) => root.columnWidthProvider(column, tableView.width)
        rowHeightProvider: function(row) {
            return root.rowHeight
        }

        model: TableModel {
            id: tableModel

            TableModelColumn { display: "name" }
            TableModelColumn { display: "fullPath" }
            TableModelColumn { display: "fileType" }
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
                    root.directorySelected(viewDelegate.fullPath)
                }
            }

            contentItem: Item {
                anchors.fill: parent

                Row {
                    id: nameColumn
                    visible: viewDelegate.column === 0
                    height: parent.height
                    anchors.left: parent.left
                    anchors.right: parent.right
                    spacing: 4

                    Item {
                        id: arrowIndentArea
                        property int depth: root.enableDirectoryNavigation ? (viewDelegate.fullPath.split("/").length - 1) : 0

                        width: root.arrowWidth + (arrowIndentArea.depth * root.indentWidth)
                        height: parent.height

                        Text {
                            anchors.left: parent.left
                            anchors.leftMargin: (arrowIndentArea.depth * root.indentWidth)
                            anchors.verticalCenter: parent.verticalCenter
                            width: root.arrowWidth
                            visible: viewDelegate.fileType === "d" && root.enableDirectoryNavigation
                            text: root._expandedDirs[viewDelegate.fullPath] === true ? "▼" : "▶"
                            color: viewDelegate.highlighted ? viewDelegate.palette.highlightedText : viewDelegate.palette.buttonText
                            verticalAlignment: Text.AlignVCenter
                        }
                    }

                    Label {
                        width: nameColumn.width - arrowIndentArea.width - nameColumn.spacing
                        text: viewDelegate.model.display
                        elide: Text.ElideRight
                        color: viewDelegate.highlighted ? viewDelegate.palette.highlightedText : viewDelegate.palette.buttonText
                        verticalAlignment: Text.AlignVCenter
                    }
                }

                Label {
                    visible: viewDelegate.column > 2
                    anchors.fill: parent
                    text: viewDelegate.model.display
                    elide: Text.ElideRight
                    color: viewDelegate.highlighted ? viewDelegate.palette.highlightedText : viewDelegate.palette.buttonText
                }
            }
        }
    }

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

    function _buildFlatList(normalizedDirPath, depth = 0) {
        let result = []
        let entries = root._cache[normalizedDirPath]
        if (!entries) {
            return result
        }
        let sortedEntries = root._sortEntries(entries)

        for (let entry of sortedEntries) {
            if (root.hideFiles && entry.fileType !== "d") {
                continue
            }
            if (root.hideDirectories && entry.fileType === "d") {
                continue
            }

            result.push(entry)

            let isExpanded = root._expandedDirs[entry.fullPath] === true
            if (entry.fileType === "d" && isExpanded) {
                let children = root._buildFlatList(entry.fullPath, depth + 1)
                result.push(...children)
            }
        }

        return result
    }

    function refreshView() {
        tableModel.clear()

        let flatList = root._buildFlatList(root.rootPath)
        for (let row of flatList) {
            tableModel.appendRow(row)
        }
    }

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

}

