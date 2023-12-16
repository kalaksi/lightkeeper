import QtQuick 2.15
import QtQuick.Controls 2.15

import "../Text"
import "../Button"
import "../js/TextTransform.js" as TextTransform
import "../js/Utils.js" as Utils


ListView {
    id: root 
    required property var rows
    property bool _invertRowOrder: true
    property string _lastQuery: ""
    property var _matchingRows: []
    property int _totalMatches: 0

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    onHeightChanged: forceLayout()
    spacing: 2
    clip: true
    focus: true
    highlightFollowsCurrentItem: true
    highlightMoveDuration: 0
    highlight: Rectangle {
        color: Theme.color_highlight_light()
    }

    model: []
    onRowsChanged: {
        if (root._invertRowOrder) {
            root.rows.reverse()
        }

        refresh()
    }


    ScrollBar.vertical: ScrollBar {
        id: scrollBar
    }

    delegate: Item {
        id: rowItem
        // property bool isRefreshButton: index === root.model.length - 1
        // height: isRefreshButton ? textContent.implicitHeight * 3 : textContent.implicitHeight
        property bool isRefreshButton: false

        width: root.width
        height: textContent.implicitHeight

        SmallText {
            id: textContent
            visible: !rowItem.isRefreshButton
            width: parent.width
            text: modelData
            font.family: "monospace"
            textFormat: Text.RichText
            wrapMode: Text.WordWrap

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                onClicked: {
                    root.currentIndex = index

                    // Right-click opens context menu.
                    if (mouse.button === Qt.RightButton) {
                        contextMenu.popup()
                    }
                }

                Menu {
                    id: contextMenu
                    MenuItem {
                        text: "Copy"
                        onTriggered: {
                            let text = root.rows[index]
                            root._copyToClipboard(text)
                        }
                    }
                }
            }
        }

        // Old implementation. Kept in case there's a need for such a button later.
        /*
        ImageButton {
            visible: rowItem.isRefreshButton
            size: textContent.implicitHeight * 2
            imageSource: "qrc:/main/images/button/refresh"
            text: "Load more"
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.verticalCenter

            onClicked: {
                let currentPage = Math.floor(root.rows.length / root.pageSize)
                root.loadMore(currentPage + 1, root.pageSize)
            }
        }
        */
    }

    Shortcut {
        sequences: [StandardKey.Copy]
        onActivated: root.copySelectionToClipboard()
    }

    // Vim-like shortcut.
    Shortcut {
        sequence: "N"
        onActivated: logList.search("down", searchField.text)
    }

    // Vim-like shortcut.
    Shortcut {
        sequence: "Shift+N"
        onActivated: logList.search("up", searchField.text)
    }

    // TODO: some UI indicator when copying happened.
    // Vim-like shortcut.
    Shortcut {
        sequence: "Y"
        onActivated: logList.copySelectionToClipboard()
    }

    // Vim-like shortcut.
    Shortcut {
        sequence: "G"
        onActivated: {
            root.currentIndex = 0
        }
    }

    // Vim-like shortcut.
    Shortcut {
        sequence: "Shift+G"
        onActivated: {
            if (root.rows.length > 0) {
                root.currentIndex = root.rows.length - 1
            }
        }
    }


    TextEdit {
        id: textEdit
        visible: false
        text: ""
    }

    function copySelectionToClipboard() {
        if (root.currentIndex >= 0) {
            let text = root.rows[root.currentIndex]
            root._copyToClipboard(text)
        }
    }

    // Workaround for copying to clipboard since there's currently no native QML way to do it (AFAIK).
    function _copyToClipboard(text) {
        textEdit.text = text
        textEdit.selectAll()
        textEdit.copy()
        console.log("Copied to clipboard: " + text)
    }

    // Could be done with rust too.
    function search(direction, query) {
        if (query !== root._lastQuery) {
            root._lastQuery = query
            refresh()
        }

        let match = -1
        if (direction === "up") {
            let reversed = [...root._matchingRows].reverse()
            match = reversed.find((row) => row < root.currentIndex)
        }
        else if (direction === "down") {
            match = root._matchingRows.find((row) => row > root.currentIndex)
        }

        if (match >= 0) {
            root.currentIndex = match
        }

        return [root._matchingRows.length, root._totalMatches]
    }

    function _newSearch(query, rows) {
        if (query.length === 0) {
            return [rows, [], 0]
        }

        let matchingRows = []
        let totalMatches = 0
        let regexp = RegExp(query, "g")

        let modelRows = []
        for (let i = 0; i < rows.length; i++) {
            let text = rows[i]
            let lastIndex = 0
            let match = regexp.exec(text)

            let rowMatches = false
            let resultRow = ""
            while (match !== null) {
                rowMatches = true
                totalMatches += 1

                let word = match[0]
                resultRow += TextTransform.escapeHtml(text.substring(lastIndex, match.index))
                resultRow += "<span style='background-color: " + Theme.color_highlight_bright() + "'>" + word + "</span>"
                lastIndex = match.index + word.length

                match = regexp.exec(text)
            }

            resultRow += TextTransform.escapeHtml(text.substring(lastIndex))
            modelRows.push(resultRow)

            if (rowMatches) {
                matchingRows.push(i)
            }
        }

        Utils.sortNumerically(matchingRows)
        return [modelRows, matchingRows, totalMatches]
    }

    function refresh() {
        let rowsClone = [...root.rows]
        let [modelRows, matchingRows, totalMatches] = _newSearch(root._lastQuery, rowsClone)
        root._matchingRows = matchingRows
        root._totalMatches = totalMatches
        /*
        if (modelRows.length > 0) {
            // Last placeholder item is reserved for "load more" button
            modelRows.push("REFRESH")
        }
        */
        root.model = modelRows
    }

    function invertRowOrder() {
        root._invertRowOrder = !root._invertRowOrder
        root.rows.reverse()
        refresh()
    }

    function addRows(newRows) {
        root.rows = newRows.concat(root.rows)
    }

    function getSearchDetails() {
        return [root._matchingRows.length, root._totalMatches]
    }

    function reset() {
        root.rows = []
        root.model = []
        root._lastQuery = ""
        root._matchingRows = []
        root._totalMatches = 0
    }

    /*
    // TODO: button for displaying only matching rows (filter locally or remotely?)
    function searchRows(query) {
        dehighlight()
        if (query.length === 0) {
            return;
        }

        CommandHandler.execute(root.hostId, root.commandId, [root._unitId, query])
    }
    */
}