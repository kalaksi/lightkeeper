import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../Text"


LightkeeperDialog {
    id: root
    modal: true
    implicitWidth: 600
    implicitHeight: 700
    standardButtons: Dialog.Close

    contentItem: ScrollView {
        id: scrollView
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        anchors.fill: parent

        ColumnLayout {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.marginScrollbar
            spacing: Theme.spacingTight

            BigText {
                text: "Keyboard shortcuts"

                Layout.alignment: Qt.AlignHCenter
                Layout.bottomMargin: Theme.spacingNormal
            }

            BigText {
                text: "Details view"
                Layout.bottomMargin: Theme.spacingNormal
            }

            Repeater {
                model: [
                    ["F5", "Refresh"],
                    ["Alt + 1, 2, 3...", "Switch tabs"],
                    ["Ctrl + W", "Close tab"],
                    ["Ctrl + T", "Open host shell in new tab\n(if linux-shell module is in use)"]
                ]

                Row {
                    NormalText {
                        width: root.implicitWidth * 0.5
                        text: modelData[0]
                    }

                    NormalText {
                        text: modelData[1]
                    }
                }
            }

            BigText {
                text: "Log viewer"

                Layout.topMargin: Theme.spacingLoose
                Layout.bottomMargin: Theme.spacingNormal
            }

            Repeater {
                model: [
                    ["Down, J", "Next line"],
                    ["Up, K", "Previous line"],
                    ["Page down", "Jump multiple lines down"],
                    ["Page up ", "Jump multiple lines up"],
                    ["Ctrl + C, Y", "Copy selected line"],
                    ["Ctrl + F, /", "Focus on search line"],
                    ["F3, N", "Next match"],
                    ["Shift + F3, Shift + N", "Previous match"],
                    ["G", "Go to top"],
                    ["Shift + G", "Go to bottom"]
                ]

                Row {
                    NormalText {
                        width: root.implicitWidth * 0.5
                        text: modelData[0]
                    }

                    NormalText {
                        text: modelData[1]
                    }
                }
            }

            BigText {
                text: "Terminal"

                Layout.topMargin: Theme.spacingLoose
                Layout.bottomMargin: Theme.spacingNormal
            }

            Repeater {
                model: [
                    ["Ctrl + F, Ctrl + Shift + F", "Search terminal buffer"],
                    ["F3", "Next match"],
                    ["Shift + F3", "Previous match"],
                    ["Ctrl + Shift + C", "Copy selection to clipboard"],
                    ["Ctrl + Shift + V", "Paste from clipboard"],
                ]

                Row {
                    NormalText {
                        width: root.implicitWidth * 0.5
                        text: modelData[0]
                    }

                    NormalText {
                        text: modelData[1]
                    }
                }
            }

        }
    }
}