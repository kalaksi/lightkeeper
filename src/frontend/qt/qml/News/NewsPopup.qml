/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import ".."
import "../Text"
import "../Misc"
import "../Button"
import "../StyleOverride" as StyleOverride


Popup {
    id: root
    property var items: []
    signal configureRequested()
    signal itemActivated(string link)

    modal: false
    focus: true
    width: 480
    padding: 0
    closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside

    // Avoid default scale/height-style enter (looks like top-to-bottom expand above the bar).
    enter: Transition {
        NumberAnimation {
            property: "opacity"
            from: 0.0
            to: 1.0
            duration: Theme.animationDurationFast
        }
    }

    exit: Transition {
        NumberAnimation {
            property: "opacity"
            from: 1.0
            to: 0.0
            duration: Theme.animationDurationFast
        }
    }

    background: BorderRectangle {
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderTop: 1
        borderBottom: 1
        borderLeft: 1
        borderRight: 1
    }

    contentItem: ColumnLayout {
        spacing: 0
        width: root.width

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: headerRow.implicitHeight + Theme.spacingNormal
            color: Theme.titleBarColor

            RowLayout {
                id: headerRow
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                anchors.leftMargin: Theme.spacingNormal
                anchors.rightMargin: Theme.spacingNormal
                spacing: Theme.spacingTight

                NormalText {
                    text: "News"
                    font.bold: true
                    Layout.fillWidth: true
                }

                Row {
                    spacing: Theme.spacingNormal

                    ImageButton {
                        flatButton: true
                        size: 22
                        imageSource: "qrc:/main/images/button/refresh"
                        tooltip: "Refresh news feeds"
                        onClicked: LK.news.refreshNow()
                    }

                    ImageButton {
                        flatButton: true
                        size: 22
                        imageSource: "qrc:/main/images/button/configure"
                        tooltip: "Configure news feeds"
                        onClicked: root.configureRequested()
                    }
                }
            }

            Rectangle {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: 1
                color: Theme.borderColor
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.preferredHeight: {
                if (root.items.length === 0) {
                    return emptyLabel.implicitHeight + Theme.spacingNormal * 2
                }
                return Math.min(360, Math.max(80, newsColumn.height)) + Theme.spacingNormal * 2
            }

            Flickable {
                id: newsList
                anchors.fill: parent
                anchors.margins: Theme.spacingNormal
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                contentWidth: width
                contentHeight: newsColumn.height
                visible: root.items.length > 0

                // Discrete wheel steps; default Flickable wheel uses kinetic flick and feels non-linear.
                WheelHandler {
                    onWheel: (event) => {
                        let maxY = Math.max(0, newsList.contentHeight - newsList.height)
                        newsList.contentY = Math.max(0, Math.min(maxY, newsList.contentY - event.angleDelta.y))
                        event.accepted = true
                    }
                }

                ScrollBar.vertical: StyleOverride.ScrollBar {
                    policy: ScrollBar.AlwaysOn
                }

                Column {
                    id: newsColumn
                    width: newsList.width - Theme.marginScrollbar
                    spacing: Theme.spacingTight

                    Repeater {
                        model: root.items

                        RowHighlight {
                            required property var modelData
                            width: newsColumn.width
                            height: itemColumn.height

                            onClicked: {
                                summaryText.text = summaryText.text !== ""
                                    ? ""
                                    : (modelData.summary || "")
                            }

                            Column {
                                id: itemColumn
                                width: parent.width

                                RowLayout {
                                    width: parent.width
                                    spacing: Theme.spacingNormal

                                    ImageButton {
                                        flatButton: true
                                        size: 28
                                        imageSource: "qrc:/main/images/button/link"
                                        tooltip: "Open in browser"
                                        enabled: !!(modelData.link && modelData.link.length > 0)
                                        onClicked: root.itemActivated(modelData.link)
                                        Layout.alignment: Qt.AlignVCenter
                                    }

                                    Column {
                                        Layout.fillWidth: true
                                        spacing: 2

                                        SmallText {
                                            width: parent.width
                                            text: modelData.title || ""
                                            elide: Text.ElideRight
                                            color: Theme.textColor
                                        }

                                        SmallerText {
                                            width: parent.width
                                            text: {
                                                let parts = []
                                                if (modelData.source) {
                                                    parts.push(modelData.source)
                                                }
                                                if (modelData.published) {
                                                    parts.push(modelData.published)
                                                }
                                                return parts.join(" - ")
                                            }
                                            elide: Text.ElideRight
                                            color: Theme.textColorDark
                                        }
                                    }
                                }

                                SmallText {
                                    id: summaryText
                                    width: parent.width
                                    visible: text !== ""
                                    wrapMode: Text.WordWrap
                                    color: Theme.textColorDark
                                    leftPadding: 28 + Theme.spacingNormal
                                    text: ""
                                }
                            }
                        }
                    }
                }
            }

            NormalText {
                id: emptyLabel
                anchors.centerIn: parent
                visible: root.items.length === 0
                text: "No news items yet"
                color: Theme.textColorDark
            }
        }
    }
}
