import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "Misc"
import "Text"


ToolBar {
    id: root
    property bool enableShortcuts: false

    signal clickedAdd()
    signal clickedRemove()
    signal clickedEdit()
    signal clickedPreferences()
    signal clickedHotkeyHelp()
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
            onClicked: root.clickedAdd()
        }

        ToolButton {
            enabled: _hostTableModel.selectedRow >= 0
            opacity: Theme.opacity(enabled)
            icon.source: "qrc:/main/images/button/remove"
            onClicked: root.clickedRemove()
        }

        ToolSeparator {
        }

        ToolButton {
            enabled: _hostTableModel.selectedRow >= 0
            opacity: Theme.opacity(enabled)
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
                    focus: true
                }
            }
        }

        ToolButton {
            icon.source: "qrc:/main/images/button/keyboard-shortcuts"
            onClicked: root.clickedHotkeyHelp()
        }

        ToolSeparator {
        }

        ToolButton {
            icon.source: "qrc:/main/images/button/configure"
            onClicked: root.clickedPreferences()
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.Find, "/"]
        onActivated: searchInput.forceActiveFocus()
    }
}