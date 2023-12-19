import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QMLTermWidget 1.0

import ".."
import "../js/Parse.js" as Parse
import "../Text"

Item {
    id: root
    property bool enableShortcuts: false

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    ColumnLayout {
        anchors.fill: parent

        QMLTermWidget {
            id: terminal
            Layout.fillWidth: true
            Layout.fillHeight: true

            font.family: "Monospace"
            font.pointSize: 10
            colorScheme: "cool-retro-term"
            smooth: true
            session: QMLTermSession {
                id: terminalSession
                initialWorkingDirectory: "$HOME"
                onMatchFound: {
                    console.log("found at: %1 %2 %3 %4".arg(startColumn).arg(startLine).arg(endColumn).arg(endLine));
                }
                onNoMatchFound: {
                    console.log("not found");
                }
            }
            QMLTermScrollbar {
                terminal: terminal
                width: 20
                Rectangle {
                    opacity: 0.4
                    anchors.margins: 5
                    radius: width * 0.5
                    anchors.fill: parent
                }
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.Find]
        onActivated: {
            searchBar.visible = true
            searchField.forceActiveFocus()
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Ctrl+Shift+C"]
        onActivated: terminal.copyClipboard()
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Ctrl+Shift+V"]
        onActivated: terminal.pasteClipboard()
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