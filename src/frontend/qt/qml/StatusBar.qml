/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "Misc"
import "Text"
import "News"
import "Dialog"
import "js/Parse.js" as Parse


ToolBar {
    id: root
    required property int errorCount
    required property int jobsLeft
    required property int hostCount
    property string remoteCoreStatus: ""
    property bool _newsPopupOpenOnPress: false
    // Keep news from overlapping side labels; use the wider side as reserve on both ends.
    readonly property real _newsMaxWidth: {
        let sideWidth = Math.max(leftSide.implicitWidth, rightSide.implicitWidth)
        return Math.max(80, root.width - sideWidth * 2 - Theme.spacingLoose * 4)
    }
    padding: 0
    leftPadding: 0
    rightPadding: 0
    topPadding: 3
    bottomPadding: 3
    implicitHeight: barContent.height + topPadding + bottomPadding

    background: BorderRectangle {
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderTop: 1
    }

    // Single content child so ToolBar does not stack the row and news overlay separately.
    Item {
        id: barContent
        width: parent.width
        height: Math.max(jobsText.implicitHeight, hostCountText.implicitHeight, newsIcon.height)

        RowLayout {
            id: statusRow
            anchors.fill: parent
            spacing: Theme.spacingLoose

            RowLayout {
                id: leftSide
                spacing: Theme.spacingLoose
                Layout.alignment: Qt.AlignVCenter

                NormalText {
                    visible: root.remoteCoreStatus.length > 0
                    leftPadding: Theme.spacingLoose * 2
                    text: root.remoteCoreStatus
                    color: Theme.textColorDark
                    elide: Text.ElideRight
                    Layout.maximumWidth: root.width * 0.25
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }

            RowLayout {
                id: rightSide
                spacing: Theme.spacingLoose
                Layout.alignment: Qt.AlignVCenter

                NormalText {
                    id: hostCountText
                    text: root.hostCount + " hosts"
                    color: Theme.textColorDark
                }

                ToolSeparator {
                    Layout.maximumHeight: barContent.height * 0.7
                    Layout.alignment: Qt.AlignVCenter
                }

                NormalText {
                    id: jobsText
                    rightPadding: Theme.spacingLoose * 2
                    text: root.jobsLeft + " jobs left"
                    color: Theme.textColorDark
                }
            }
        }

        // Centered on the full bar so remote-core text on the left does not shift it.
        Item {
            id: newsControl
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.verticalCenter: parent.verticalCenter
            width: newsTitle.visible
                ? newsIcon.width + Theme.spacingNormal + newsTitle.width
                : newsIcon.width
            height: Math.max(newsIcon.height, newsTitle.visible ? newsTitle.implicitHeight : 0)
            z: 1

            OverlayImage {
                id: newsIcon
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                width: 16
                height: 16
                source: "qrc:/main/images/button/rss"
                color: Theme.iconColor
            }

            NormalText {
                id: newsTitle
                anchors.left: newsIcon.right
                anchors.leftMargin: Theme.spacingNormal
                anchors.verticalCenter: parent.verticalCenter
                visible: text.length > 0
                width: visible
                    ? Math.min(implicitWidth, root._newsMaxWidth - newsIcon.width - Theme.spacingNormal)
                    : 0
                text: LK.news.latestTitle
                color: Theme.textColorDark
                elide: Text.ElideRight

                onTextChanged: {
                    if (text.length === 0) {
                        newsPulse.stop()
                        newsTitle.color = Theme.textColorDark
                        newsIcon.color = Theme.iconColor
                        return
                    }
                    newsPulse.restart()
                }
            }

            MouseArea {
                anchors.fill: parent
                cursorShape: Qt.PointingHandCursor
                // Capture before CloseOnPressOutside dismisses the popup on this same click.
                onPressed: root._newsPopupOpenOnPress = newsPopup.visible
                onClicked: {
                    if (root._newsPopupOpenOnPress) {
                        newsPopup.close()
                        return
                    }
                    newsPopup.items = Parse.TryParseJson(LK.news.getItemsJson()) || []
                    newsPopup.open()
                }
            }
        }
    }

    SequentialAnimation {
        id: newsPulse
        loops: 3

        ParallelAnimation {
            ColorAnimation {
                target: newsTitle
                property: "color"
                to: "#ffffff"
                duration: 180
            }

            ColorAnimation {
                target: newsIcon
                property: "color"
                to: "#ffffff"
                duration: 180
            }
        }

        ParallelAnimation {
            ColorAnimation {
                target: newsTitle
                property: "color"
                to: Theme.textColorDark
                duration: 180
            }

            ColorAnimation {
                target: newsIcon
                property: "color"
                to: Theme.iconColor
                duration: 180
            }
        }
    }

    NewsPopup {
        id: newsPopup

        // Reposition after first layout; avoid tracking height changes (that animated top-to-bottom).
        onOpened: Qt.callLater(root.repositionNewsPopup)

        onConfigureRequested: {
            newsPopup.close()
            newsConfigDialog.open()
        }

        onItemActivated: (link) => Qt.openUrlExternally(link)
    }

    NewsConfigDialog {
        id: newsConfigDialog
        anchors.centerIn: Overlay.overlay
    }

    function repositionNewsPopup() {
        let newsPos = newsControl.mapToItem(root, 0, 0)
        let preferredX = newsPos.x + (newsControl.width - newsPopup.width) / 2
        newsPopup.x = Math.max(0, Math.min(preferredX, root.width - newsPopup.width))
        newsPopup.y = -newsPopup.height - 4
    }
}
