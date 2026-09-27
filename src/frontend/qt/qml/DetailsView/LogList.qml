/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls

import Lightkeeper 1.0

import "../js/Utils.js" as Utils
import "../StyleOverride"


ListView {
    id: root 
    property var rows: []
    property bool enableShortcuts: true
    property string selectionColor: Theme.highlightColorLight.toString()
    property string searchText: ""
    property bool invertRowOrder: true
    /// If enabled, only appends new rows to the model instead of reprocessing all. Makes processing more performant.
    /// Not compatible with invertRowOrder as new rows are always appended to the end.
    property bool appendOnly: false
    property string _lastQuery: ""
    property var _matchingRows: []
    property int _totalMatches: 0
    property int _listPageSize: 15
    /// For appendOnly mode: keeps track of received rows so only new rows are appended.
    property int _lastRowCount: 0
    /// Sparse set of selected model indices. Reassign the object when mutating so bindings update.
    property var _selectedIndices: ({})
    property int _selectedCount: 0
    property int _selectionAnchor: -1

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    onHeightChanged: forceLayout()
    spacing: 2
    clip: true
    focus: true
    reuseItems: true
    highlightFollowsCurrentItem: false

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
        required property int index
        required property string text

        id: rowItem
        width: root.width - scrollBar.width
        height: textContent.implicitHeight
        // Depend on _selectedCount so reused delegates refresh when the set changes.
        property bool selected: root._selectedCount >= 0 && root._selectedIndices[rowItem.index] === true

        // Drop leftover selection when the delegate is recycled.
        ListView.onPooled: textContent.deselect()
        ListView.onReused: textContent.deselect()

        Rectangle {
            anchors.fill: parent
            color: rowItem.selected ? root.selectionColor : "transparent"
        }

        TextEdit {
            id: textContent
            width: parent.width
            text: rowItem.text || ""
            color: Theme.textColor
            font.family: "monospace"
            font.pointSize: Theme.fontSize - 2
            textFormat: TextEdit.RichText
            wrapMode: TextEdit.Wrap
            readOnly: true
            selectByMouse: true
            selectByKeyboard: false
            persistentSelection: true
            activeFocusOnPress: true
            cursorVisible: false

            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.LeftButton | Qt.RightButton
                cursorShape: Qt.IBeamCursor
                onPressed: function(mouse) {
                    if (mouse.button === Qt.RightButton) {
                        if (root._selectedIndices[rowItem.index] !== true) {
                            root.selectOnly(rowItem.index)
                        }
                        mouse.accepted = true
                        return
                    }

                    if (mouse.modifiers & Qt.ShiftModifier) {
                        root.selectRange(rowItem.index)
                        mouse.accepted = true
                        return
                    }

                    if (mouse.modifiers & Qt.ControlModifier) {
                        root.toggleSelect(rowItem.index)
                        mouse.accepted = true
                        return
                    }

                    // Plain click/drag: mark the line, then let TextEdit handle text selection.
                    root.selectOnly(rowItem.index)
                    mouse.accepted = false
                }
                onClicked: function(mouse) {
                    if (mouse.button === Qt.RightButton) {
                        contextMenu.popup()
                    }
                }

                Menu {
                    id: contextMenu
                    MenuItem {
                        text: "Copy"
                        onTriggered: root.copySelectionToClipboard()
                    }
                }
            }
        }

        function hasTextSelection() {
            return textContent.selectedText.length > 0
        }

        function copyTextSelection() {
            textContent.copy()
        }

        function clearTextSelection() {
            textContent.deselect()
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
        onActivated: root.search("down", root.searchText)
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Shift+N"
        onActivated: root.search("up", root.searchText)
    }

    // TODO: some UI indicator when copying happened.
    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Y"
        onActivated: root.copySelectionToClipboard()
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "G"
        onActivated: root.selectOnly(0)
    }

    // Vim-like shortcut.
    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Shift+G"
        onActivated: {
            if (root.rows.length > 0) {
                root.selectOnly(root.rows.length - 1)
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.MoveToPreviousLine, "K"]
        onActivated: {
            root.decrementCurrentIndex()
            root.selectOnly(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.MoveToNextLine, "J"]
        onActivated: {
            root.incrementCurrentIndex()
            root.selectOnly(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Shift+Up", "Shift+K"]
        onActivated: {
            root.decrementCurrentIndex()
            root.selectRange(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Shift+Down", "Shift+J"]
        onActivated: {
            root.incrementCurrentIndex()
            root.selectRange(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: StandardKey.MoveToPreviousPage
        onActivated: {
            root.currentIndex -= Math.min(root._listPageSize, root.currentIndex)
            root.selectOnly(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: StandardKey.MoveToNextPage
        onActivated: {
            root.currentIndex += Math.min(root._listPageSize, root.count - root.currentIndex)
            root.selectOnly(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Shift+PgUp"
        onActivated: {
            root.currentIndex -= Math.min(root._listPageSize, root.currentIndex)
            root.selectRange(root.currentIndex)
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Shift+PgDown"
        onActivated: {
            root.currentIndex += Math.min(root._listPageSize, root.count - root.currentIndex)
            root.selectRange(root.currentIndex)
        }
    }


    TextEdit {
        id: textEdit
        visible: false
        text: ""
    }

    function selectOnly(modelIndex) {
        if (modelIndex < 0) {
            return
        }
        // Drop partial text highlights on other rows so only one selection is visible.
        root._clearAllTextSelections()
        let selected = {}
        selected[modelIndex] = true
        root._selectedIndices = selected
        root._selectedCount = 1
        root._selectionAnchor = modelIndex
        root.currentIndex = modelIndex
    }

    function toggleSelect(modelIndex) {
        if (modelIndex < 0) {
            return
        }
        root._clearAllTextSelections()
        let selected = Object.assign({}, root._selectedIndices)
        if (selected[modelIndex] === true) {
            delete selected[modelIndex]
        }
        else {
            selected[modelIndex] = true
        }
        root._selectedIndices = selected
        root._selectedCount = Object.keys(selected).length
        root._selectionAnchor = modelIndex
        root.currentIndex = modelIndex
    }

    function selectRange(modelIndex) {
        if (modelIndex < 0) {
            return
        }
        root._clearAllTextSelections()
        let anchor = root._selectionAnchor >= 0 ? root._selectionAnchor : modelIndex
        let top = Math.min(anchor, modelIndex)
        let bottom = Math.max(anchor, modelIndex)
        let selected = {}
        for (let i = top; i <= bottom; i++) {
            selected[i] = true
        }
        root._selectedIndices = selected
        root._selectedCount = bottom - top + 1
        root.currentIndex = modelIndex
    }

    function clearSelection() {
        root._clearAllTextSelections()
        root._selectedIndices = {}
        root._selectedCount = 0
        root._selectionAnchor = -1
    }

    function copySelectionToClipboard() {
        // Prefer in-line text selection when only one line is selected.
        if (root._selectedCount <= 1) {
            let item = root.itemAtIndex(root.currentIndex)
            if (item && item.hasTextSelection()) {
                item.copyTextSelection()
                return
            }
        }

        let indices = Object.keys(root._selectedIndices).map(Number)
        if (indices.length === 0 && root.currentIndex >= 0) {
            indices = [root.currentIndex]
        }
        if (indices.length === 0) {
            return
        }

        Utils.sortNumerically(indices)
        let lines = indices.map((modelIndex) => {
            let index = root.invertRowOrder ? root.rows.length - 1 - modelIndex : modelIndex
            return root.rows[index]
        })
        root._copyToClipboard(lines.join("\n"))
    }

    function _clearAllTextSelections() {
        for (let i = 0; i < root.count; i++) {
            let item = root.itemAtIndex(i)
            if (item) {
                item.clearTextSelection()
            }
        }
    }

    // Workaround for copying to clipboard since there's currently no native QML way to do it (AFAIK).
    function _copyToClipboard(text) {
        textEdit.text = text
        textEdit.selectAll()
        textEdit.copy()
        console.log("Copied to clipboard: " + text)
    }

    // TODO: Use Rust model instead?
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
            root.selectOnly(match)
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
        let highlightOpen = "<span style='background-color: " + Theme.highlightColorBright + "'>"
        let highlightClose = "</span>"

        let modelRows = []
        for (let i = 0; i < rows.length; i++) {
            // Rows are already Qt rich text (journalctl/ANSI spans). Search only text segments so
            // existing markup stays intact and tags are not matched or escaped into visible text.
            let resultRow = ""
            let rowMatches = false
            for (const part of rows[i].split(/(<[^>]+>)/)) {
                if (part.startsWith("<") && part.endsWith(">")) {
                    resultRow += part
                    continue
                }

                regexp.lastIndex = 0
                let lastIndex = 0
                let match = regexp.exec(part)
                while (match !== null) {
                    let word = match[0]
                    if (word.length === 0) {
                        regexp.lastIndex += 1
                        match = regexp.exec(part)
                        continue
                    }

                    rowMatches = true
                    totalMatches += 1
                    resultRow += part.substring(lastIndex, match.index)
                    resultRow += highlightOpen + word + highlightClose
                    lastIndex = match.index + word.length
                    match = regexp.exec(part)
                }

                resultRow += part.substring(lastIndex)
            }

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
            else if (root.model.count > 0) {
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
            let rowsClone = [...root.rows]
            if (root.invertRowOrder) {
                rowsClone.reverse()
            }

            let [modelRows, matchingRows, totalMatches] = _newSearch(root._lastQuery, rowsClone)
            root._matchingRows = matchingRows
            root._totalMatches = totalMatches

            root.model.clear()
            for (const row of modelRows ) {
                root.model.append({"text": row})
            }

            if (root.model.count === 0) {
                root.clearSelection()
            }
            else if (root.currentIndex >= 0 && root.currentIndex < root.model.count) {
                root.selectOnly(root.currentIndex)
            }
            else {
                root.selectOnly(0)
            }
        }
    }

    function toggleInvertRowOrder() {
        root.invertRowOrder = !root.invertRowOrder
        refresh()

        if (root.rows.length > 0) {
            if (root.invertRowOrder) {
                root.selectOnly(0)
            }
            else {
                root.selectOnly(root.rows.length - 1)
            }
        }
    }

    function getSearchDetails() {
        return [root._matchingRows.length, root._totalMatches]
    }

    function resetFields() {
        root.model.clear()
        root.rows = []
        root._lastQuery = ""
        root._matchingRows = []
        root._totalMatches = 0
        root._lastRowCount = 0
        root.clearSelection()
    }
}