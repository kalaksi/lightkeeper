import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "Misc"
import "Text"


ToolBar {
    id: root
    property bool enableShortcuts: false
    property bool enableEditButtons: false

    focus: true


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
        anchors.fill: parent

        ToolButton {
            icon.source: "qrc:/main/images/button/add"
            text: "Add host"
            display: AbstractButton.IconOnly
            onClicked: root.clickedAdd()
        }

        ToolButton {
            enabled: root.enableEditButtons
            opacity: Theme.opacity(enabled)
            text: "Remove host"
            display: AbstractButton.IconOnly
            icon.source: "qrc:/main/images/button/remove"
            onClicked: root.clickedRemove()
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
                    placeholderText: "by name or address..."
                    width: root.width * 0.4
                    onTextChanged: root.filterChanged(searchInput.text)
                }
            }
        }

        ToolButton {
            icon.source: "qrc:/main/images/button/certificates"
            text: "Cert. monitor"
            display: AbstractButton.TextBesideIcon
            onClicked: root.clickedCertificateMonitor()
        }

        ToolSeparator { }

        ToolButton {
            icon.source: "qrc:/main/images/button/keyboard-shortcuts"
            text: "Keyboard shortcuts"
            display: AbstractButton.IconOnly
            onClicked: root.clickedHotkeyHelp()
        }

        ToolSeparator { }

        ToolButton {
            icon.source: "qrc:/main/images/button/configure"
            text: "Configuration"
            display: AbstractButton.IconOnly
            onClicked: root.clickedPreferences()
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.Find, "/"]
        onActivated: searchInput.forceActiveFocus()
    }
}