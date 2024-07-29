import QtQuick 2.15
import QtQuick.Controls 2.15

import "../Text"
import "../Button"
import "../js/TextTransform.js" as TextTransform
import "../js/Utils.js" as Utils


ListView {
    id: root 
    required property var rows
    property bool enableShortcuts: true
    property string selectionColor: Theme.highlightColorLight
    property bool invertRowOrder: true
    property bool scrollToBottom: !invertRowOrder
    property string _lastQuery: ""
    property var _matchingRows: []
    property int _totalMatches: 0
    property int _listPageSize: 15

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
        color: root.selectionColor
    }

    model: []
    onRowsChanged: {
        if (root.invertRowOrder) {
            root.rows.reverse()
        }

        refresh()
    }


    ScrollBar.vertical: ScrollBar {
        id: scrollBar
    }

    delegate: Item {
        id: rowItem
        width: root.width - scrollBar.width
        height: textContent.implicitHeight

        SmallText {
            id: textContent
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
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.Copy]
        onActivated: root.copySelectionToClipboard()
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "N"
        onActivated: logList.search("down", searchField.text)
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Shift+N"
        onActivated: logList.search("up", searchField.text)
    }

    // TODO: some UI indicator when copying happened.
    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Y"
        onActivated: logList.copySelectionToClipboard()
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "G"
        onActivated: {
            root.currentIndex = 0
        }
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Shift+G"
        onActivated: {
            if (root.rows.length > 0) {
                root.currentIndex = root.rows.length - 1
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.MoveToPreviousLine, "K"]
        onActivated: decrementCurrentIndex()
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.MoveToNextLine, "J"]
        onActivated: incrementCurrentIndex()
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: StandardKey.MoveToPreviousPage
        onActivated: {
            root.currentIndex -= Math.min(root._listPageSize, root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: StandardKey.MoveToNextPage
        onActivated: {
            root.currentIndex += Math.min(root._listPageSize, root.count - root.currentIndex)
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

    // TODO: Use rust model instead?
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
                resultRow += "<span style='background-color: " + Theme.highlightColorBright + "'>" + word + "</span>"
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
        root.model = modelRows

        if (root.scrollToBottom) {
            root.currentIndex = root.rows.length - 1
            // root.currentIndex = Math.min(root.currentIndex, root.rows.length - 1)
        }
    }

    function invertRowOrder() {
        root.invertRowOrder = !root.invertRowOrder
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
}