/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Text"
import "../Button"
import "../Misc"
import "../StyleOverride"
import "../js/Parse.js" as Parse


LightkeeperDialog {
    id: root
    title: "News feeds"
    implicitWidth: 720
    implicitHeight: 460
    standardButtons: Dialog.Ok | Dialog.Cancel
    property int nameFieldWidth: 220
    property int buttonSize: 32

    property var _config: ({ feeds: [] })

    onAboutToShow: {
        root._config = Parse.TryParseJson(LK.config.getNewsConfigJson()) || { feeds: [] }
        enabledCheck.checked = root._config.enabled === true
        intervalField.text = String(root._config.refresh_interval || 0)
        feedModel.clear()
        for (const feed of (root._config.feeds || [])) {
            feedModel.append({
                name: feed.name || "",
                url: feed.url || "",
                enabled: feed.enabled !== false
            })
        }
    }

    onAccepted: {
        let feeds = []
        for (let i = 0; i < feedModel.count; i++) {
            const row = feedModel.get(i)
            if (!row.url || !row.url.trim()) {
                continue
            }
            feeds.push({
                name: (row.name && row.name.trim()) ? row.name.trim() : row.url.trim(),
                url: row.url.trim(),
                enabled: !!row.enabled
            })
        }
        const config = {
            enabled: enabledCheck.checked === true,
            refresh_interval: Math.max(0, parseInt(intervalField.text, 10) || 0),
            feeds: feeds
        }
        if (!LK.config.setNewsConfigJson(JSON.stringify(config))) {
            return
        }
        LK.applyNewsConfig()
    }

    contentItem: ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        ColumnLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingNormal

            CheckBox {
                id: enabledCheck
                text: "Enable news"
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingNormal

                NormalText {
                    text: "Refresh interval"
                    color: Theme.textColorDark
                }

                TextField {
                    id: intervalField
                    Layout.preferredWidth: 90
                    validator: IntValidator { bottom: 0 }
                }

                NormalText {
                    text: "seconds (0 disables automatic polling)"
                    color: Theme.textColorDark
                }

                Item { Layout.fillWidth: true }
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.topMargin: Theme.spacingLoose
            spacing: Theme.spacingNormal

            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.spacingNormal

                // Match checkbox column width so headers line up with fields.
                Item {
                    Layout.preferredWidth: 28
                }

                SmallerText {
                    Layout.preferredWidth: root.nameFieldWidth
                    text: "Name"
                    color: Theme.textColorDark
                }

                SmallerText {
                    Layout.fillWidth: true
                    text: "URL"
                    color: Theme.textColorDark
                }

                Item {
                    Layout.preferredWidth: 28
                }
            }

            ListView {
                id: feedList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                spacing: Theme.spacingNormal
                model: ListModel { id: feedModel }

                delegate: RowLayout {
                    width: feedList.width
                    spacing: Theme.spacingNormal
                    height: 36

                    CheckBox {
                        Layout.preferredWidth: 28
                        Layout.alignment: Qt.AlignVCenter
                        checked: model.enabled
                        onToggled: feedModel.setProperty(index, "enabled", checked)
                    }

                    TextField {
                        Layout.preferredWidth: root.nameFieldWidth
                        Layout.alignment: Qt.AlignVCenter
                        text: model.name
                        placeholderText: "Name"
                        onEditingFinished: feedModel.setProperty(index, "name", text)
                    }

                    TextField {
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                        text: model.url
                        placeholderText: "Feed URL"
                        onEditingFinished: feedModel.setProperty(index, "url", text)
                    }

                    ImageButton {
                        Layout.preferredWidth: 28
                        Layout.alignment: Qt.AlignVCenter
                        flatButton: true
                        size: 22
                        imageSource: "qrc:/main/images/button/delete"
                        tooltip: "Remove feed"
                        onClicked: feedModel.remove(index)
                    }
                }
            }

            ImageButton {
                flatButton: false
                size: root.buttonSize
                imageSource: "qrc:/main/images/button/add"
                text: "Add feed"
                onClicked: feedModel.append({ name: "", url: "", enabled: true })
            }
        }
    }
}
