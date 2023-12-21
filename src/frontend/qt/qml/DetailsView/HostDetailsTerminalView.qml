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
                let newResults = [..._searchMatches, {
                    "startColumn": startColumn,
                    "startLine": startLine,
                    "endColumn": endColumn,
                    "endLine": endLine
                }]

                // Forces re-evaluation of the Repeater
                _searchMatches = newResults

                scrollbar.value = endLine
            }
            onNoMatchFound: {
                console.log("not found");
            }
        }

        /*
        QMLTermScrollbar {
            id: scrollbar
            terminal: terminal
            width: 20
            opacity: 1.0

            Rectangle {
                opacity: 0.4
                anchors.margins: 5
                radius: width * 0.5
                anchors.fill: parent

            }

            MouseArea {
                anchors.fill: parent
                drag.target: parent
                drag.axis: Drag.YAxis
                drag.minimumY: 0
                drag.maximumY: terminal.height - parent.height
                drag.minimumX: 0
                drag.maximumX: 0
                drag.smoothed: false
            }
        }
        */

        // Tried QMLTermScrollbar, but functionality was lacking.
        ScrollBar {
            id: scrollbar
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 20
            opacity: 1.0
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
        }
    }

    Repeater {
        anchors.fill: terminal
        id: matchHighlights
        model: root._searchMatches

        // TODO: multi-line highlighting
        delegate: Rectangle {
            color: Theme.highlightColorBrighter
            opacity: 0.5
            x: terminal.fontMetrics.width * modelData.startColumn
            y: terminal.fontMetrics.height * modelData.startLine + terminal.lineSpacing
            height: terminal.fontMetrics.height
            width: terminal.fontMetrics.width * (modelData.endColumn - modelData.startColumn)

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
                onClicked: findPrevious.trigger()
            }

            ImageButton {
                flatButton: true
                imageSource: "qrc:/main/images/button/search-down"
                onClicked: findNext.trigger()
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
            searchField.forceActiveFocus()
        }
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
            terminalSession.search(searchField.text)
            terminal.updateImage()
        }
    }

    Action {
        id: findPrevious
        enabled: root.enableShortcuts
        shortcut: StandardKey.FindPrevious
        onTriggered: {
            terminal.forceActiveFocus()
            terminalSession.search(searchField.text, 0, 0, true)
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

    function focus() {
        terminal.forceActiveFocus()
        root.enableShortcuts = true
    }

    function unfocus() {
        root.enableShortcuts = false
    }

    function close() {
        terminalSession.sendSignal(15)
        terminalSession.clearScreen()
    }

    function refresh()  {
        terminalSession.clearScreen()
    }
}