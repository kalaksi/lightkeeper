import QtQuick 2.15
import QtQuick.Controls 2.15

Item {
    id: root
    width: 600
    height: 400

    // Recursive content property:
    // {
    //   "dir1": {
    //     "file1": ["col1", "col2"]
    //   },
    //   "file2": ["col1", "col2"]
    // }
    property var content: ({})
    property int indentWidth: 20
    property int rowHeight: 28
    property int arrowWidth: 20
    property int nameColumnWidth: 200
    property var columnHeaders: ["Column 1", "Column 2"]
    property var columnWidths: [200, 200]
    property color headerColor: "#333333"
    property color dirColor: "#eeeeee"
    property color fileColor: "#ffffff"
    property color borderColor: "#cccccc"
    property color hoverColor: "#e0e0e0"

    // Callback for directory expansion
    property var onDirectoryExpanded: function(path) {}
    // Callback for file/directory click
    property var onItemClicked: function(path, isDirectory) {}

    // Helper function to check if value is a directory (object, not array)
    function isDirectory(value) {
        if (!value || typeof value !== 'object')
            return false
        // In QML, arrays are objects, so we need to check for array specifically
        return !Array.isArray(value)
    }

    // Calculate total width of data columns
    readonly property int dataColumnsWidth: {
        var total = 0
        for (var i = 0; i < columnWidths.length; i++)
            total += columnWidths[i]
        return total
    }

    Flickable {
        anchors.fill: parent
        contentWidth: parent.width
        contentHeight: treeColumn.implicitHeight
        clip: true

        Column {
            id: treeColumn
            width: parent.width

            // Header
            Rectangle {
                width: parent.width
                height: rowHeight
                color: headerColor
                Row {
                    anchors.fill: parent
                    anchors.leftMargin: arrowWidth
                    spacing: 4
                    Text {
                        width: nameColumnWidth
                        text: "Name"
                        color: "white"
                        verticalAlignment: Text.AlignVCenter
                        leftPadding: 4
                    }
                    Repeater {
                        model: columnHeaders.length
                        Text {
                            width: columnWidths[index]
                            text: columnHeaders[index]
                            color: "white"
                            verticalAlignment: Text.AlignVCenter
                            leftPadding: 4
                        }
                    }
                }
            }

            // Content
            Loader {
                width: parent.width
                active: root.content && Object.keys(root.content).length > 0
                sourceComponent: treeNode
                onLoaded: {
                    item.modelObject = root.content
                    item.level = 0
                    item.parentPath = ""
                    item.pathStack = []
                }
            }
        }
    }

    Component {
        id: treeNode

        Column {
            id: nodeRoot
            property var modelObject
            property int level
            property string parentPath: ""
            property var pathStack: []
            width: parent.width

            Repeater {
                model: modelObject ? Object.keys(modelObject) : []

                delegate: Item {
                    id: rowItem
                    width: nodeRoot.width
                    height: nodeContent.implicitHeight

                    property string keyName: modelData
                    property var valueObject: modelObject[keyName]
                    property bool isDir: root.isDirectory(valueObject)
                    property bool expanded: false
                    property bool hovered: false

                    Column {
                        id: nodeContent
                        width: parent.width

                        Rectangle {
                            width: parent.width
                            height: rowHeight
                            color: {
                                if (hovered)
                                    return hoverColor
                                return rowItem.isDir ? dirColor : fileColor
                            }
                            border.color: borderColor
                            border.width: 1

                            Row {
                                anchors.fill: parent
                                anchors.leftMargin: 4
                                spacing: 4

                                Item {
                                    width: level * indentWidth
                                    height: 1
                                }

                                // Expand / collapse indicator
                                Text {
                                    width: arrowWidth
                                    text: rowItem.isDir ? (rowItem.expanded ? "▼" : "▶") : ""
                                    verticalAlignment: Text.AlignVCenter
                                    horizontalAlignment: Text.AlignHCenter
                                }

                                Text {
                                    width: nameColumnWidth
                                    text: keyName
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                }

                                Repeater {
                                    model: root.columnWidths.length
                                    Text {
                                        width: root.columnWidths[index]
                                        text: rowItem.isDir ? "" : (valueObject && valueObject[index] !== undefined ? valueObject[index] : "")
                                        verticalAlignment: Text.AlignVCenter
                                        elide: Text.ElideRight
                                    }
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                hoverEnabled: true
                                onEntered: rowItem.hovered = true
                                onExited: rowItem.hovered = false
                                onClicked: {
                                    // Build full path
                                    let pathParts = nodeRoot.pathStack.concat([keyName])
                                    let fullPath
                                    if (pathParts.length === 0 || (pathParts.length === 1 && pathParts[0] === "")) {
                                        fullPath = "/"
                                    } else {
                                        fullPath = "/" + pathParts.join("/")
                                    }

                                    if (rowItem.isDir) {
                                        let wasExpanded = rowItem.expanded
                                        rowItem.expanded = !rowItem.expanded
                                        
                                        // If expanding and directory is empty, trigger callback
                                        if (rowItem.expanded && !wasExpanded && Object.keys(valueObject).length === 0) {
                                            if (root.onDirectoryExpanded) {
                                                root.onDirectoryExpanded(fullPath)
                                            }
                                        }
                                    } else {
                                        // File clicked
                                        if (root.onItemClicked) {
                                            root.onItemClicked(fullPath, false)
                                        }
                                    }
                                }
                            }
                        }

                        // Recursive children
                        Loader {
                            visible: rowItem.isDir && rowItem.expanded
                            width: parent.width
                            active: visible
                            sourceComponent: treeNode
                            onLoaded: {
                                item.modelObject = rowItem.isDir ? valueObject : ({})
                                item.level = nodeRoot.level + 1
                                item.parentPath = nodeRoot.parentPath ? nodeRoot.parentPath + "/" + keyName : keyName
                                item.pathStack = nodeRoot.pathStack.concat([keyName])
                            }
                        }
                    }
                }
            }
        }
    }
}
