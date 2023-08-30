import QtQuick 2.15
import QtQuick.Controls 1.4
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Button"
import "../Text"
import "../js/Utils.js" as Utils
import ".."

// This component should be a direct child of main window.
Dialog {
    id: root
    property var _preferences: ConfigManager.get_preferences()
    property bool _loading: true

    modal: true
    implicitWidth: 550
    implicitHeight: 650
    background: DialogBackground { }
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged()

    onOpened: {
        root._preferences = ConfigManager.get_preferences()
        root._loading = false
    }

    onAccepted: {
        let newPreferences = {
            refresh_hosts_on_start: content.children[1].children[1].checkState === Qt.Checked,
            use_remote_editor: content.children[2].children[1].checkState === Qt.Checked,
            remote_text_editor: content.children[3].children[1].text,
            sudo_remote_editor: content.children[4].children[1].checkState === Qt.Checked,
            text_editor: content.children[5].children[1].text,
            terminal: content.children[6].children[1].text.split(" ")[0],
            terminal_args: content.children[6].children[1].text.split(" ").slice(1).join(" ")
        }

        ConfigManager.set_preferences(newPreferences)
        root._loading = true
        root.configurationChanged()
    }

    onRejected: {
        root._loading = true
    }

    Item {
        visible: root._loading
        Layout.fillWidth: true
        Layout.fillHeight: true

        WorkingSprite {
        }
    }

    contentItem: ColumnLayout {
        id: content
        visible: !root._loading
        anchors.margins: Theme.margin_dialog()
        anchors.bottomMargin: Theme.margin_dialog_bottom()
        spacing: Theme.spacing_loose()

        BigText {
            text: "Preferences"

            Layout.alignment: Qt.AlignHCenter
        }

        RowLayout {
            Layout.fillWidth: true

            Column {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Label {
                    width: parent.width
                    text: "Refresh hosts on start"
                }

                SmallText {
                    width: parent.width
                    text: "Refresh all hosts on application start"
                    color: Theme.color_dark_text()
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                checkState: root._preferences.refresh_hosts_on_start ? Qt.Checked : Qt.Unchecked
            }
        }

        RowLayout {
            Layout.fillWidth: true

            Column {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Label {
                    width: parent.width
                    text: "Use remote editor"
                }

                SmallText {
                    width: parent.width
                    text: "Some commands allow you to edit a remote file. If checked, will launch a terminal for editing " +
                          "directly on the remote host instead of first downloading a local copy."
                    color: Theme.color_dark_text()
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                id: useRemoteEditor
                checkState: root._preferences.use_remote_editor ? Qt.Checked : Qt.Unchecked
            }
        }

        RowLayout {
            Layout.fillWidth: true

            Label {
                text: "Remote text editor"
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignTop
            }

            TextField {
                enabled: useRemoteEditor.checkState === Qt.Checked
                text: root._preferences.remote_text_editor
            }
        }

        RowLayout {
            Layout.fillWidth: true

            Column {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Label {

                    width: parent.width
                    text: "Use sudo with remote editor"
                }

                SmallText {
                    width: parent.width
                    text: "Use sudo when editing remote files?"
                    color: Theme.color_dark_text()
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                enabled: useRemoteEditor.checkState === Qt.Checked
                checkState: root._preferences.sudo_remote_editor ? Qt.Checked : Qt.Unchecked
            }
        }

        RowLayout {
            Layout.fillWidth: true

            Column {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Label {
                    width: parent.width
                    text: "Local text editor"
                }

                SmallText {
                    width: parent.width
                    text: "The text editor to use when editing files locally."
                    color: Theme.color_dark_text()
                    wrapMode: Text.WordWrap
                }
            }

            TextField {
                text: root._preferences.text_editor
            }
        }

        RowLayout {
            Layout.fillWidth: true

            Column {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Label {
                    width: parent.width
                    text: "Terminal"
                }

                SmallText {
                    width: parent.width
                    text: "Terminal to use when launching a remote shell."
                    color: Theme.color_dark_text()
                    wrapMode: Text.WordWrap
                }
            }

            TextField {
                text: root._preferences.terminal + " " + root._preferences.terminal_args
            }
        }

        NormalText {
            text: "Accepting changes will reload the application."
            color: Theme.color_dark_text()
            wrapMode: Text.Wrap
            width: root._contentWidth

            Layout.alignment: Qt.AlignHCenter
        }
    }
}