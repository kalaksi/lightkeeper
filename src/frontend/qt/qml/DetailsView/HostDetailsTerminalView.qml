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


    function start(command) {
        terminalSession.setShellProgram(command[0])
        terminalSession.setArgs(command.slice(1))
        terminalSession.startShellProgram()
        terminal.forceActiveFocus()
    }

    function end() {
        terminalSession.sendSignal(15)
        terminalSession.clearScreen()
    }
}