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

            // News is centered in the stretch slot; long titles elide up to the slot width.
            Item {
                id: centerFill
                Layout.fillWidth: true
                Layout.fillHeight: true

                Item {
                    id: newsControl
                    anchors.centerIn: parent
                    width: newsIcon.width
                        + (newsTitle.visible ? Theme.spacingNormal + newsTitle.width : 0)
                    height: Math.max(newsIcon.height, newsTitle.implicitHeight)

                    OverlayImage {
                        id: newsIcon
                        anchors.left: parent.left
                        anchors.verticalCenter: parent.verticalCenter
                        width: 16
                        height: 16
                        source: "qrc:/main/images/button/rss"
                        color: Theme.iconColor
                    }

                    // Unelided twin for natural width; elide+width on the same Text collapses to 0.
                    NormalText {
                        id: newsTitleMeasure
                        visible: false
                        text: newsTitle.text
                    }

                    NormalText {
                        id: newsTitle
                        anchors.left: newsIcon.right
                        anchors.leftMargin: Theme.spacingNormal
                        anchors.verticalCenter: parent.verticalCenter
                        visible: text.length > 0
                        width: Math.min(
                            newsTitleMeasure.implicitWidth,
                            Math.max(0, centerFill.width - newsIcon.width - Theme.spacingNormal)
                        )
                        text: LK.news.latestTitle
                        color: Theme.textColorDark
                        elide: Text.ElideRight

                        onTextChanged: (text) => {
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
        // Popup positions with x/y relative to parent (no anchors).
        parent: newsControl
        x: (parent.width - width) / 2
        y: -height - 4

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
}
