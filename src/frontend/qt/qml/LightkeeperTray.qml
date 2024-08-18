import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11
import Qt.labs.platform 1.1



SystemTrayIcon {
    id: root
    property int normals: 0
    property int warnings: 0
    property int errors: 0

    visible: true
    icon.source: "qrc:/main/images/tray-icon"
    tooltip: "Lightkeeper: " + normals + " normal, " + warnings + " warning, " + errors + " error"

    signal showClicked()
    signal quitClicked()

    onActivated: function(reason) {
        if (reason === SystemTrayIcon.Context) {
            trayMenu.open()
        }
        else if (reason === SystemTrayIcon.DoubleClick) {
            root.showClicked()
        }
        else if (reason === SystemTrayIcon.Trigger) {
            root.showClicked()
        }
        else if (reason === SystemTrayIcon.MiddleClick) {
            root.quitClicked()
        }
    }

    menu: Menu {
        id: trayMenu
        visible: false

        MenuItem {
            text: "Show / hide"
            onTriggered: root.showClicked()
        }

        MenuSeparator { }

        MenuItem {
            enabled: false
            text: "Error: " + root.errors
        }

        MenuItem {
            enabled: false
            text: "Warning: " + root.warnings
        }

        MenuItem {
            enabled: false
            text: "Normal: " + root.normals
        }

        MenuSeparator { }

        MenuItem {
            text: "Quit"
            onTriggered: root.quitClicked()
        }
    }
}