/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

import Lightkeeper 1.0

import "../Text"
import "../Misc"
import ".."
import "../StyleOverride"

LightkeeperDialog {
    id: root
    property var _preferences: LK.config.getPreferences()
    property bool _loading: true

    title: "Preferences"
    implicitWidth: 650
    implicitHeight: 800
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal configurationChanged()

    onOpened: {
        root._preferences = LK.config.getPreferences()
        root._loading = false
        tabBar.currentIndex = 0

        // Update ComboBox when preferences are loaded
        Qt.callLater(function() {
            if (textEditorCombo) {
                textEditorOther.text = root._preferences.textEditor || ""
            }
        })
    }

    onAccepted: {
        let newPreferences = {
            refreshHostsOnStart: refreshHostsOnStart.checkState === Qt.Checked,
            useRemoteEditor: useRemoteEditor.checkState === Qt.Checked,
            remoteTextEditor: remoteTextEditor.text,
            sudoRemoteEditor: useSudoRemoteEditor.checkState === Qt.Checked,
            textEditor: (() => {
                let value = textEditorCombo._getValue(textEditorCombo.currentIndex)
                return value === "other" ? (textEditorOther.text || "internal") : value
            })(),
            terminal: terminalAndArgs.text.split(" ")[0],
            terminalArgs: terminalAndArgs.text.split(" ").slice(1).join(" "),
            showStatusBar: showStatusBar.checkState === Qt.Checked,
            showCharts: showCharts.checkState === Qt.Checked,
            closeToTray: closeToTray.checkState === Qt.Checked,
            showMonitorNotifications: showMonitorNotifications.checkState === Qt.Checked,
        }

        LK.config.setPreferences(newPreferences)
        root._loading = true
        root.configurationChanged()
    }

    onRejected: {
        root._loading = true
    }

    Shortcut {
        enabled: root.visible
        sequences: ["Alt+1", "Ctrl+1"]
        onActivated: tabBar.currentIndex = 0
    }

    Shortcut {
        enabled: root.visible
        sequences: ["Alt+2", "Ctrl+2"]
        onActivated: tabBar.currentIndex = 1
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
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: 0

        TabBar {
            id: tabBar
            currentIndex: 0
            contentHeight: 36

            Layout.fillWidth: true

            background: Item {
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    height: 1
                    color: Theme.borderColor
                }
            }

            LKTabButton {
                text: "Settings"
                active: tabBar.currentIndex === 0
            }

            LKTabButton {
                text: "About"
                active: tabBar.currentIndex === 1
            }
        }

        StackLayout {
            currentIndex: tabBar.currentIndex

            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.topMargin: Theme.spacingLoose

            //
            // "Settings" tab
            //

            ColumnLayout {
                spacing: Theme.spacingLoose

                RowLayout {
                    Layout.fillWidth: true

                    Column {
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter

                        Label {
                            width: parent.width
                            text: "Close to tray"
                        }

                        SmallText {
                            width: parent.width
                            text: ""
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }
                    }

                    CheckBox {
                        id: closeToTray
                        checkState: root._preferences.closeToTray ? Qt.Checked : Qt.Unchecked

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
                            text: "Show monitor notifications"
                        }

                        SmallText {
                            width: parent.width
                            text: "When monitor state changes to either critical, error, or warning, a notification will be shown in the system tray."
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }
                    }

                    CheckBox {
                        id: showMonitorNotifications
                        checkState: root._preferences.showMonitorNotifications ? Qt.Checked : Qt.Unchecked

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
                    visible: useRemoteEditor.checkState === Qt.Checked

                    Label {
                        text: "Remote text editor"

                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignTop
                    }

                    TextField {
                        id: remoteTextEditor
                        enabled: useRemoteEditor.checkState === Qt.Checked
                        text: root._preferences.remoteTextEditor
                        validator: RegularExpressionValidator {
                            regularExpression: /^[a-zA-Z0-9_\-\.\/\s]+$/
                        }

                        Layout.preferredWidth: content.width * 0.35
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    visible: useRemoteEditor.checkState === Qt.Checked

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
                            text: LK.config.isSandboxed() ?
                                  "The text editor to use when editing files locally. Only integrated editors are available in flatpak." :
                                  "The text editor to use when editing files locally. Integrated editor is always used with flatpak."
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }
                    }

                    ComboBox {
                        id: textEditorCombo

                        model: ListModel {
                            ListElement { display: "internal"; value: "internal" }
                            ListElement { display: "internal (simple)"; value: "internal-simple" }
                            ListElement { display: "Other..."; value: "other" }
                        }

                        textRole: "display"

                        delegate: ItemDelegate {
                            width: textEditorCombo.width
                            text: model.display
                            enabled: !LK.config.isSandboxed() || model.value !== "other"
                        }

                        function _getValue(index) {
                            return model.get(index).value
                        }

                        function _determineCurrentIndex() {
                            let textEditor = root._preferences.textEditor || "internal"

                            if (textEditor === "internal") {
                                return 0
                            } else if (textEditor === "internal-simple") {
                                return 1
                            } else {
                                return 2
                            }
                        }

                        Component.onCompleted: {
                            currentIndex = _determineCurrentIndex()
                            // If "Other..." is selected but sandboxed, default to "internal"
                            if (currentIndex === 2) {
                                if (LK.config.isSandboxed()) {
                                    currentIndex = 0
                                } else {
                                    textEditorOther.text = root._preferences.textEditor || ""
                                }
                            }
                        }

                        Layout.preferredWidth: content.width * 0.35
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    visible: textEditorCombo.currentIndex === 2

                    Item {
                        Layout.fillWidth: true
                    }

                    TextField {
                        id: textEditorOther
                        enabled: !LK.config.isSandboxed()
                        placeholderText: "execute..."
                        validator: RegularExpressionValidator {
                            regularExpression: /^[a-zA-Z0-9_\-\.\/\s]+$/
                        }

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
                            text: "Terminal to use when launching a remote shell. Integrated editor is always used with flatpak."
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }
                    }

                    TextField {
                        id: terminalAndArgs
                        text: root._preferences.terminal +
                            (root._preferences.terminalArgs.length > 0 ? " " + root._preferences.terminalArgs.join(" ") : "")
                        enabled: !LK.config.isSandboxed()
                        validator: RegularExpressionValidator {
                            regularExpression: /^(internal|[a-zA-Z0-9_\-\.\/\s]+)$/
                        }

                        Layout.preferredWidth: content.width * 0.35
                    }
                }

                RowLayout {
                    // TODO: remove later if not needed?
                    visible: false

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
                        enabled: false
                        opacity: 0.5
                        checkState: root._preferences.showStatusBar ? Qt.Checked : Qt.Unchecked

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
                            text: "Enable charts (experimental, requires restart)"
                        }

                        SmallText {
                            width: parent.width
                            text: "Enables charts, alert history, and local metrics server for historical data."
                            color: Theme.textColorDark
                            wrapMode: Text.WordWrap
                        }
                    }

                    CheckBox {
                        id: showCharts
                        checkState: root._preferences.showCharts ? Qt.Checked : Qt.Unchecked

                        Layout.leftMargin: content.width * 0.30
                    }
                }

                // Spacer
                Item {
                    Layout.fillHeight: true
                }
            }

            //
            // "About" tab
            //

            ColumnLayout {
                spacing: Theme.spacingNormal

                BigText {
                    text: "Lightkeeper"
                    Layout.alignment: Qt.AlignHCenter
                }

                NormalText {
                    text: "Version " + LK.config.getVersion()
                    Layout.alignment: Qt.AlignHCenter
                }

                SmallText {
                    text: "A customizable server management tool for maintaining servers over SSH."
                    color: Theme.textColorDark
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter

                    Layout.fillWidth: true
                    Layout.topMargin: Theme.spacingLoose
                    Layout.leftMargin: Theme.spacingLoose
                    Layout.rightMargin: Theme.spacingLoose
                }

                SmallText {
                    text: "© 2023 kalaksi@users.noreply.github.com"
                    color: Theme.textColorDark
                    horizontalAlignment: Text.AlignHCenter

                    Layout.fillWidth: true
                    Layout.topMargin: Theme.spacingLoose
                }

                SmallText {
                    text: "Licensed under GNU General Public License 3 or later."
                    color: Theme.textColorDark
                    horizontalAlignment: Text.AlignHCenter

                    Layout.fillWidth: true
                }

                NormalText {
                    text: '<a href="https://github.com/kalaksi/lightkeeper">https://github.com/kalaksi/lightkeeper</a>'
                    textFormat: Text.RichText
                    horizontalAlignment: Text.AlignHCenter
                    Layout.fillWidth: true
                    Layout.topMargin: Theme.spacingNormal
                    onLinkActivated: (link) => Qt.openUrlExternally(link)

                    MouseArea {
                        anchors.fill: parent
                        acceptedButtons: Qt.NoButton
                        cursorShape: parent.hoveredLink ? Qt.PointingHandCursor : Qt.ArrowCursor
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }
    }
}
