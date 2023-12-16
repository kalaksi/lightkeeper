import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../Text"


// This component should be a direct child of main window.
Dialog {
    id: root
    modal: true
    opacity: 0.0
    // implicitWidth: dialogText.implicitWidth + 100
    // implicitHeight: dialogText.implicitHeight + 100
    implicitWidth: 500
    implicitHeight: 550
    standardButtons: Dialog.Close

    background: DialogBackground { }


    onVisibleChanged: {
        if (visible) {
            root.opacity = 1.0
        }
        else {
            root.opacity = 0.0
        }
    }

    ScrollView {
        id: scrollView
        anchors.fill: parent

        ColumnLayout {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.margin_scrollbar()
            spacing: Theme.spacing_tight()

            BigText {
                text: "Keyboard shortcuts"

                Layout.alignment: Qt.AlignHCenter
                Layout.bottomMargin: Theme.spacingNormal
            }

            BigText {
                text: "Details view"
                Layout.bottomMargin: Theme.spacingTight
            }

            Repeater {
                model: [
                    ["F5", "Refresh"],
                    ["Alt + 1, 2, 3...", "Switch tabs"],
                    ["Ctrl + w", "Close tab"]
                ]

                Row {
                    NormalText {
                        width: root.implicitWidth * 0.55
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
                    ["Down, j", "Next line"],
                    ["Up, k", "Previous line"],
                    ["Page down", "Jump multiple lines down"],
                    ["Page up ", "Jump multiple lines up"],
                    ["Ctrl + c, y", "Copy selected line"],
                    ["Ctrl + f, /", "Focus on search line"],
                    ["F3, n", "Next match"],
                    ["Shift + F3, N", "Previous match"],
                    ["g", "Go to top"],
                    ["G", "Go to bottom"]
                ]

                Row {
                    NormalText {
                        width: root.implicitWidth * 0.55
                        text: modelData[0]
                    }

                    NormalText {
                        text: modelData[1]
                    }
                }
            }

        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animationDuration
        }
    }
}