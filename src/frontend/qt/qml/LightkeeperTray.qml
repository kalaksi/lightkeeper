import QtQuick
import QtQuick.Controls
import Qt.labs.platform



SystemTrayIcon {
    id: root
    property int criticalCount: 0
    property int errorCount: 0
    property int warningCount: 0
    property int normalCount: 0
    property int nodataCount: 0

    visible: true
    icon.source: "qrc:/main/images/tray-icon"
    tooltip: {
        let texts = [];

        if (root.criticalCount > 0) {
            texts.push("Critical: " + root.criticalCount)
        }
        if (root.errorCount > 0) {
            texts.push("Error: " + root.errorCount)
        }
        if (root.warningCount > 0) {
            texts.push("Warning: " + root.warningCount)
        }
        if (root.normalCount > 0) {
            texts.push("Normal: " + root.normalCount)
        }
        if (root.nodataCount > 0) {
            texts.push("No Data: " + root.nodataCount)
        }

        return "Lightkeeper status:\n" + texts.join(", ")
    }

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
            text: "Critical: " + root.criticalCount
        }

        MenuItem {
            enabled: false
            text: "Error: " + root.errorCount
        }

        MenuItem {
            enabled: false
            text: "Warning: " + root.warningCount
        }

        MenuItem {
            enabled: false
            text: "Normal: " + root.normalCount
        }

        MenuItem {
            enabled: false
            text: "No Data: " + root.nodataCount
        }

        MenuSeparator { }

        MenuItem {
            text: "Quit"
            onTriggered: root.quitClicked()
        }
    }
}