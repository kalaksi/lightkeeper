import QtQuick 2.15
import QtQuick.Controls 2.15

import "../Text"
import "../Button"
import "../js/TextTransform.js" as TextTransform
import "../js/Utils.js" as Utils


ListView {
    id: root 
    property var rows: []
    property bool enableShortcuts: true
    property string selectionColor: Theme.highlightColorLight
    property bool invertRowOrder: true
    property bool scrollToBottom: !invertRowOrder
    /// If enabled, only appends new rows to the model instead of reprocessing all. Makes processing more performant.
    /// Not compatible with invertRowOrder as new rows are always appended to the end.
    property bool appendOnly: false
    property string _lastQuery: ""
    property var _matchingRows: []
    property int _totalMatches: 0
    property int _listPageSize: 15
    /// For appendOnly mode: keeps track of received rows so only new rows are appended.
    property int _lastRowCount: 0

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

    model: ListModel {
        id: listModel
    }

    onRowsChanged: {
        refresh()
    }

    ScrollBar.vertical: ScrollBar {
        id: scrollBar
        policy: ScrollBar.AlwaysOn
    }

    delegate: Item {
        id: rowItem
        width: root.width - scrollBar.width
        height: textContent.implicitHeight

        SmallText {
            id: textContent
            width: parent.width
            text: modelData || ""
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
        if (root.appendOnly) {
            let newRows = []
            if (root._lastRowCount === 0) {
                newRows = root.rows
            }
            else {
                // Last line may be partial so replace that too.
                root.model.remove(root.model.count - 1)
                newRows = root.rows.slice(root._lastRowCount - 1)
            }

            root._lastRowCount = root.rows.length

            for (const row of newRows) {
                root.model.append({"text": row})
            }
        }
        else {
            if (root.invertRowOrder) {
                root.rows.reverse()
            }

            let rowsClone = [...root.rows]
            let [modelRows, matchingRows, totalMatches] = _newSearch(root._lastQuery, rowsClone)
            root._matchingRows = matchingRows
            root._totalMatches = totalMatches

            root.model.clear()
            for (const row of modelRows) {
                root.model.append({"text": row})
            }
        }

        if (root.scrollToBottom) {
            root.positionViewAtEnd()
        }
    }

    function toggleInvertRowOrder() {
        root.invertRowOrder = !root.invertRowOrder
        root.rows.reverse()
        refresh()
    }

    function getSearchDetails() {
        return [root._matchingRows.length, root._totalMatches]
    }

    function reset() {
        root.rows = []
        root.model.clear()
        root._lastQuery = ""
        root._matchingRows = []
        root._totalMatches = 0
        root._lastRowCount = 0
    }
}