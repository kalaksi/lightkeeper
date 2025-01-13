import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Button"
import "../Text"
import ".."

LightkeeperDialog {
    id: root
    property string hostId: ""
    property bool _loading: hostId === ""

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    title: `Custom commands`
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configSaved(string hostId)

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: scrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root._loading
    }

    contentItem: ScrollView {
        id: scrollView
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        contentWidth: availableWidth
        clip: true

        Column {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.marginScrollbar
            spacing: Theme.spacingNormal

            Repeater {
                id: repeater
                model: []

                RowLayout {
                    id: rowLayout
                    width: parent.width
                    height: textContainer.implicitHeight
                    spacing: Theme.spacingNormal

                    Column {
                        id: textContainer
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter

                        Label {
                            width: parent.width
                            text: modelData.key
                        }

                        SmallText {
                            width: parent.width
                            text: modelData.description
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }

                        TextField {
                            id: textField
                            enabled: toggleSwitch.checked && !fileChooserButton.visible
                            placeholderText: toggleSwitch.checked ? "" : "unset"
                            placeholderTextColor: Theme.textColorDark
                            text: toggleSwitch.checked ? modelData.value : ""

                            Layout.preferredWidth: {
                                if (fileChooserButton.visible) {
                                    scrollView.width * 0.35 - fileChooserButton.width - rowLayout.spacing
                                }
                                else {
                                    scrollView.width * 0.35
                                }
                            }
                            Layout.alignment: Qt.AlignVCenter

                            Connections {
                                target: DesktopPortal
                                function onFileChooserResponse(token, filePath) {
                                    if (fileChooserButton.visible && token === fileChooserButton._fileChooserToken) {
                                        textField.text = filePath
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}