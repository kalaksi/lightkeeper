import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QMLTermWidget 1.0

import ".."
import "../Button"
import "../js/Parse.js" as Parse
import "../Text"

Item {
    id: root
    property bool enableShortcuts: false
    property var _searchMatches: []


    Component.onCompleted: {
        _searchMatches = []
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    QMLTermWidget {
        id: terminal
        anchors.fill: parent

        font.family: "Monospace"
        font.pointSize: 10
        colorScheme: "cool-retro-term"
        smooth: true
        session: QMLTermSession {
            id: terminalSession
            initialWorkingDirectory: "$HOME"
            onMatchFound: function(startColumn, startLine, endColumn, endLine) {
                // Doesn't currently append results since only match can currently be shown at once.
                let newResult = [{
                    "startColumn": startColumn,
                    "startLine": startLine,
                    "endColumn": endColumn,
                    "endLine": endLine
                }]

                // Forces re-evaluation of the Repeater
                root._searchMatches = newResult

                let lineOnScreen = startLine - terminal.scrollbarCurrentValue

                // Is not on current screen, scrolling needed.
                if (lineOnScreen < 0 || lineOnScreen > terminal.lines) {
                    terminal.scrollToLine(startLine)
                }

                terminal.scrollToLine(startLine)
            }
        }

        // Tried QMLTermScrollbar, but functionality was lacking.
        ScrollBar {
            id: scrollbar
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            active: true
            orientation: Qt.Vertical
            size: {
                let totalRows = terminal.lines + terminal.scrollbarMaximum
                return terminal.lines / totalRows
            }
            position: {
                let freeScrollable = 1.0-size
                return (terminal.scrollbarCurrentValue / terminal.scrollbarMaximum) * freeScrollable
            }

            Connections {
                target: scrollbar

                function onPositionChanged() {
                    if (scrollbar.pressed) {
                        terminal.scrollToPosition(scrollbar.position)
                    }
                }
            }
        }

        /// This is a hack. Couldn't figure out a better way to control the current position of terminal buffer.
        /// Not fully accurate so sometimes line is a few rows outside of view.
        function scrollToLine(line) {
            // Limit iterations just in case.
            for (let i = 0; i < 100; i++) {
                let deltaY = (terminal.scrollbarCurrentValue - line)
                if ((deltaY >= 0 && deltaY < 1) || (deltaY < 0 && deltaY > -1)) {
                    break
                }

                terminal.simulateWheel(0, deltaY, 0, 0, Qt.point(0, deltaY))
            }
        }

        function scrollToPosition(position) {
            let freeScrollable = 1.0-scrollbar.size
            let newScrollValue = (position / freeScrollable) * terminal.scrollbarMaximum
            scrollToLine(newScrollValue)
        }
    }

    Repeater {
        id: matchHighlights
        anchors.fill: terminal
        model: root._searchMatches

        // TODO: multi-line highlighting for long lines
        delegate: Rectangle {
            property int lineOnScreen: modelData.startLine - terminal.scrollbarCurrentValue

            visible: lineOnScreen >= 0 && lineOnScreen < terminal.lines
            color: Theme.highlightColorBright
            x: terminal.fontMetrics.width * modelData.startColumn
            y: terminal.fontMetrics.height * lineOnScreen + 2
            height: terminal.fontMetrics.height
            // + 1 since it doesn't cover all characters otherwise
            width: terminal.fontMetrics.width * (modelData.endColumn - modelData.startColumn + 1)
        }
    }

    Rectangle {
        id: searchBar
        visible: false
        color: Theme.backgroundColorLight
        height: searchField.height + Theme.spacingNormal * 2 - 4
        width: 340
        anchors.top: parent.top
        anchors.right: parent.right
        anchors.rightMargin: 30

        RowLayout {
            anchors.fill: parent
            anchors.margins: 5
            spacing: Theme.spacingTight

            TextField {
                id: searchField
                width: parent.width * 0.5
                placeholderText: "Search"
                onAccepted: findNext.trigger()
            }

            ImageButton {
                flatButton: true
                imageSource: "qrc:/main/images/button/search-up"
                onClicked: findNext.trigger()
            }

            ImageButton {
                flatButton: true
                imageSource: "qrc:/main/images/button/search-down"
                onClicked: findPrevious.trigger()
            }

            // Spacer
            Item {
                Layout.fillWidth: true
            }

            ImageButton {
                flatButton: true
                imageSource: "qrc:/main/images/button/tab-close"
                onClicked: closeSearch.trigger()
            }
        }
    }

    Action {
        id: openSearch
        enabled: root.enableShortcuts
        shortcut: StandardKey.Find
        onTriggered: {
            searchBar.visible = true
            searchField.selectAll()
            searchField.forceActiveFocus()
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Ctrl+Shift+F"
        onActivated: openSearch.trigger()

    }

    Action {
        id: closeSearch
        enabled: root.enableShortcuts
        shortcut: StandardKey.Cancel
        onTriggered: {
            root._searchMatches = []
            searchBar.visible = false
            terminal.forceActiveFocus()
        }
    }

    Action {
        id: findNext
        enabled: root.enableShortcuts
        shortcut: StandardKey.FindNext
        onTriggered: {
            terminal.forceActiveFocus()
            let currentMatch = root._searchMatches[0]
            if (currentMatch !== undefined) {
                terminalSession.search(searchField.text, currentMatch.startLine, currentMatch.startColumn, false)
            } else {
                let end = terminal.lines + terminal.scrollbarCurrentValue
                terminalSession.search(searchField.text, end, 0, false)
            }
            terminal.updateImage()
        }
    }

    Action {
        id: findPrevious
        enabled: root.enableShortcuts
        shortcut: StandardKey.FindPrevious
        onTriggered: {
            terminal.forceActiveFocus()

            let currentMatch = root._searchMatches[0]
            if (currentMatch !== undefined) {
                terminalSession.search(searchField.text, currentMatch.endLine, currentMatch.endColumn)
            } else {
                terminalSession.search(searchField.text)
            }

            terminal.updateImage()
        }
    }

    Action {
        enabled: root.enableShortcuts
        shortcut: "Ctrl+Shift+C"
        onTriggered: terminal.copyClipboard()
    }

    Action {
        enabled: root.enableShortcuts
        shortcut: "Ctrl+Shift+V"
        onTriggered: terminal.pasteClipboard()
    }


    function open(command) {
        terminalSession.setShellProgram(command[0])
        terminalSession.setArgs(command.slice(1))
        terminalSession.startShellProgram()
    }

    function close() {
        terminalSession.sendSignal(15)
        terminalSession.clearScreen()
    }

    function focus() {
        terminal.forceActiveFocus()
        root.enableShortcuts = true
    }

    function unfocus() {
        root.enableShortcuts = false
    }

    function refresh()  {
        terminalSession.clearScreen()
    }
}