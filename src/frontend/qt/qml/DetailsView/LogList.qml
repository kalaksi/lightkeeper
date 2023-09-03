import QtQuick 2.15
import QtQuick.Controls 2.15

import "../Text"
import "../Button"
import "../js/TextTransform.js" as TextTransform


ListView {
    id: root 
    required property var rows
    property int pageSize: 400
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
    onRowsChanged: refreshModel()

    signal loadMore(int pageNumber, int pageSize)



    ScrollBar.vertical: ScrollBar {
        id: scrollBar
    }

    Shortcut {
        sequence: [
            StandardKey.Copy,
            "Ctrl+C",
            StandardKey.Cancel
        ]
        onActivated: {
            if (root.currentIndex >= 0) {
                let text = root.rows[root.currentIndex]
                root.copyToClipboard(text)
            }
        }
    }

    delegate: Item {
        id: rowItem
        property bool isRefreshButton: index === root.model.length - 1

        width: root.width
        height: isRefreshButton ? textContent.implicitHeight * 3 : textContent.implicitHeight

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
                            root.copyToClipboard(text)
                        }
                    }
                }
            }
        }

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
    }

    TextEdit {
        id: textEdit
        visible: false
        text: ""
    }

    // Workaround for copying to clipboard since there's currently no native QML way to do it (AFAIK).
    function copyToClipboard(text) {
        textEdit.text = text
        textEdit.selectAll()
        textEdit.copy()
        console.log("Copied to clipboard: " + text)
    }

    // Could be done with rust too.
    function search(direction, query) {
        if (query !== root._lastQuery) {
            console.log("NEWSEARCH")
            root._lastQuery = query

            let [highlightedRows, matchingRows, totalMatches] = _newSearch(query, root.rows)
            console.log("rows", root.rows.length, "hilites", highlightedRows.length, "matches", matchingRows.length, "total", totalMatches)
            console.log(JSON.stringify(highlightedRows))
            root._matchingRows = matchingRows
            root._totalMatches = totalMatches
            refreshModel()
        }

        let match = -1
        if (direction === "up") {
            let reversed = [...root._matchingRows].reverse()
            match = reversed.find((row) => root.currentIndex > row)
        }
        else if (direction === "down") {
            match = root._matchingRows.find((row) => root.currentIndex < row)
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

        let highlightedRows = []
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
            highlightedRows.push(resultRow)

            if (rowMatches) {
                matchingRows.push(i)
            }
        }

        return [highlightedRows, matchingRows, totalMatches]
    }

    function refreshModel() {
        let rowsClone = [...root.rows]
        let [modelRows, matchingRows, totalMatches] = _newSearch(root._lastQuery, rowsClone)
        root._matchingRows = matchingRows
        root._totalMatches = totalMatches
        modelRows.reverse()
        if (modelRows.length > 0) {
            // Last placeholder item is reserved for "load more" button
            modelRows.push("REFRESH")
        }
        root.model = modelRows
    }

    function addRows(newRows) {
        root.rows = newRows.filter((row) => row.length > 0).concat(root.rows)
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
    // TODO
    function searchRows(query) {
        dehighlight()
        if (query.length === 0) {
            return;
        }

        CommandHandler.execute(root.hostId, root.commandId, [root._unitId, query])
    }
    */
}