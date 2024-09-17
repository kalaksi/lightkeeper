import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "Misc"
import "Text"
import "Button"


ToolBar {
    id: root
    property bool enableShortcuts: false
    property bool enableEditButtons: false
    property int refreshProgress: 100
    property int iconSize: 24

    focus: true
    height: 38

    signal clickedAdd()
    signal clickedRemove()
    signal clickedEdit()
    signal clickedPreferences()
    signal clickedHotkeyHelp()
    signal clickedCertificateMonitor()
    signal filterChanged(string search)

    background: BorderRectangle {
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderBottom: 1
    }

    RowLayout {
        width: parent.width
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingNormal

        ToolButton {
            icon.source: "qrc:/main/images/button/add"
            text: "Add host"
            display: AbstractButton.IconOnly
            onClicked: root.clickedAdd()
            icon.height: root.iconSize
            icon.width: root.iconSize
            padding: 4
        }

        ToolButton {
            enabled: root.enableEditButtons
            opacity: Theme.opacity(enabled)
            text: "Remove host"
            display: AbstractButton.IconOnly
            icon.source: "qrc:/main/images/button/remove"
            onClicked: root.clickedRemove()
            icon.height: root.iconSize
            icon.width: root.iconSize
            padding: 4
        }

        ToolSeparator {
        }

        ToolButton {
            enabled: root.enableEditButtons
            opacity: Theme.opacity(enabled)
            display: AbstractButton.IconOnly
            text: "Edit host"
            icon.source: "qrc:/main/images/button/entry-edit"
            onClicked: root.clickedEdit()
            icon.height: root.iconSize
            icon.width: root.iconSize
            padding: 4
        }

        Item {
            Layout.fillWidth: true

            Row {
                spacing: Theme.spacingLoose
                anchors.centerIn: parent

                Label {
                    text: "Search:"
                    anchors.verticalCenter: parent.verticalCenter
                }

                TextField {
                    id: searchInput
                    anchors.verticalCenter: parent.verticalCenter
                    placeholderText: "by name or address..."
                    // TODO: color from global theme / palette? by default it's too dark.
                    placeholderTextColor: Theme.textColorDark
                    width: root.width * 0.3
                    onTextChanged: root.filterChanged(searchInput.text)
                }

                AutoRefreshButton {
                    enabled: root.refreshProgress === 100
                    spinning: root.refreshProgress < 100
                    size: certMonitorButton.height
                    onClicked: {
                        LK.command.forceInitializeHosts()
                    }
                }
            }
        }


        ToolButton {
            id: certMonitorButton
            icon.source: "qrc:/main/images/button/certificates"
            text: "Cert. monitor"
            display: AbstractButton.TextBesideIcon
            onClicked: root.clickedCertificateMonitor()
            icon.height: root.iconSize
            icon.width: root.iconSize
            padding: 4
        }

        ToolSeparator { }

        ToolButton {
            icon.source: "qrc:/main/images/button/keyboard-shortcuts"
            text: "Keyboard shortcuts"
            display: AbstractButton.IconOnly
            onClicked: root.clickedHotkeyHelp()
            icon.height: root.iconSize
            icon.width: root.iconSize
            padding: 4
            topPadding: 2
            bottomPadding: 6
        }

        ToolButton {
            icon.source: "qrc:/main/images/button/configure"
            text: "Configuration"
            display: AbstractButton.IconOnly
            onClicked: root.clickedPreferences()
            icon.height: root.iconSize
            icon.width: root.iconSize
            padding: 4
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.Find, "/"]
        onActivated: searchInput.forceActiveFocus()
    }
}