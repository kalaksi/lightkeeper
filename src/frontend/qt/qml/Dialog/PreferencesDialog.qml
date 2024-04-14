import QtQuick 2.15
import QtQuick.Layouts 1.11
import QtQuick.Controls 1.4
import QtQuick.Controls 2.15

import "../StyleOverride"
import "../Button"
import "../Text"
import "../js/Utils.js" as Utils
import ".."

// This component should be a direct child of main window.
LightkeeperDialog {
    id: root
    property var _preferences: ConfigManager.getPreferences()
    property bool _loading: true

    title: "Preferences"
    implicitWidth: 550
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged()

    onOpened: {
        root._preferences = ConfigManager.getPreferences()
        root._loading = false
    }

    onAccepted: {
        let newPreferences = {
            refreshHostsOnStart: refreshHostsOnStart.checkState === Qt.Checked,
            useRemoteEditor: useRemoteEditor.checkState === Qt.Checked,
            remoteTextEditor: remoteTextEditor.text,
            sudoRemoteEditor: useSudoRemoteEditor.checkState === Qt.Checked,
            textEditor: textEditor.text,
            terminal: terminalAndArgs.text.split(" ")[0],
            terminalArgs: terminalAndArgs.text.split(" ").slice(1).join(" "),
            showStatusBar: showStatusBar.checkState === Qt.Checked,
        }

        ConfigManager.setPreferences(newPreferences)
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
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

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
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                id: refreshHostsOnStart
                checkState: root._preferences.refreshHostsOnStart ? Qt.Checked : Qt.Unchecked

                Layout.leftMargin: content.width * 0.30
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
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                id: useRemoteEditor
                checkState: root._preferences.useRemoteEditor ? Qt.Checked : Qt.Unchecked

                Layout.leftMargin: content.width * 0.30
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
                id: remoteTextEditor
                enabled: useRemoteEditor.checkState === Qt.Checked
                text: root._preferences.remoteTextEditor

                Layout.preferredWidth: content.width * 0.35
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
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                id: useSudoRemoteEditor
                enabled: useRemoteEditor.checkState === Qt.Checked
                checkState: root._preferences.sudoRemoteEditor ? Qt.Checked : Qt.Unchecked

                Layout.leftMargin: content.width * 0.30
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
                    text: "The text editor to use when editing files locally. Integrated editor is always used in flatpak sandbox."
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            TextField {
                id: textEditor
                text: root._preferences.textEditor
                enabled: !ConfigManager.isSandboxed()

                Layout.preferredWidth: content.width * 0.35
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
                    text: "Terminal to use when launching a remote shell. Integrated editor is always used in flatpak sandbox."
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            TextField {
                id: terminalAndArgs
                text: root._preferences.terminal + " " + root._preferences.terminalArgs
                enabled: !ConfigManager.isSandboxed()

                Layout.preferredWidth: content.width * 0.35
            }
        }

        RowLayout {
            Layout.fillWidth: true

            Column {
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter

                Label {
                    width: parent.width
                    text: "Show status bar"
                }

                SmallText {
                    width: parent.width
                    text: ""
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                }
            }

            CheckBox {
                id: showStatusBar
                checkState: root._preferences.showStatusBar ? Qt.Checked : Qt.Unchecked

                Layout.leftMargin: content.width * 0.30
            }
        }

        // Spacer
        Item {
            Layout.fillHeight: true
        }
    }
}