import QtQuick 2.15
import QtQuick.Controls 2.15

import "../Text"
import "../js/TextTransform.js" as TextTransform


ListView {
    id: root 
    required property var rows
    property string _lastQuery: ""
    property var _matchingRows: []

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

    model: rows.reverse()


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

    delegate: SmallText {
        width: root.width
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
            root._lastQuery = query
            root._matchingRows = []

            if (query.length === 0) {
                root.model = root.rows.reverse()
                return;
            }

            let regexp = RegExp(query, "g")

            let result = []
            for (let i = 0; i < root.rows.length; i++) {
                let text = root.rows[i]
                let lastIndex = 0
                let match = regexp.exec(text)

                let rowMatches = false
                let resultRow = ""
                while (match !== null) {
                    rowMatches = true
                    let word = match[0]
                    resultRow += TextTransform.escapeHtml(text.substring(lastIndex, match.index))
                    resultRow += "<span style='background-color: " + Theme.color_highlight_bright() + "'>" + word + "</span>"
                    lastIndex = match.index + word.length

                    match = regexp.exec(text)
                }

                resultRow += TextTransform.escapeHtml(text.substring(lastIndex))
                result.push(resultRow)

                if (rowMatches) {
                    root._matchingRows.push(i)
                }
            }

            root.model = result
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