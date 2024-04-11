import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "Misc"
import "Text"


ToolBar {
    id: root

    signal clickedAdd()
    signal clickedRemove()
    signal clickedEdit()
    signal clickedPreferences()
    signal clickedHotkeyHelp()

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


        /* TODO: implement later?
        Row {
            id: searchRow
            spacing: Theme.spacing_loose()

            Layout.fillWidth: true
            Layout.leftMargin: Theme.spacing_loose() * 4

            Label {
                text: "Search:"
                anchors.verticalCenter: parent.verticalCenter
            }

            TextField {
                id: searchInput
                text: "Search by name or address"

                width: parent.width * 0.5
            }
        }
        */

        // Spacer
        Item {
            Layout.fillWidth: true
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
}