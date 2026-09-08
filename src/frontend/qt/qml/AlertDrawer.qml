/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "Text"
import "Misc"
import "Button"
import "StyleOverride" as StyleOverride


Item {
    id: root
    property bool open: false
    property var alerts: []

    readonly property var unacked: alerts.filter(alert => !alert.acknowledged)
    readonly property var acked: alerts.filter(alert => alert.acknowledged)
    readonly property int panelWidth: Math.min(460, Math.max(360, parent ? parent.width * 0.36 : 460))

    signal closeRequested()
    signal alertActivated(var alert)

    visible: open || panel.x < width

    // Dimmer over main content.
    Rectangle {
        anchors.fill: parent
        color: "#99000000"
        opacity: root.open ? 1 : 0
        visible: opacity > 0.01

        Behavior on opacity {
            NumberAnimation {
                duration: Theme.animationDuration
                easing.type: Easing.OutQuad
            }
        }

        MouseArea {
            anchors.fill: parent
            onClicked: root.closeRequested()
        }
    }

    BorderRectangle {
        id: panel
        width: root.panelWidth
        height: parent.height
        x: root.open ? parent.width - width : parent.width
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderLeft: 1
        clip: true
        focus: root.open

        Behavior on x {
            NumberAnimation {
                duration: Theme.animationDuration
                easing.type: Easing.OutQuad
            }
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.spacingNormal
            anchors.rightMargin: Theme.spacingNormal
            anchors.bottomMargin: Theme.spacingNormal
            // No top margin here — title row provides matching top/bottom padding.
            spacing: 0

            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: Theme.spacingNormal
                Layout.bottomMargin: Theme.spacingNormal
                spacing: Theme.spacingNormal

                OverlayImage {
                    source: "qrc:/main/images/button/alerts"
                    color: Theme.iconColor
                    Layout.preferredWidth: 20
                    Layout.preferredHeight: 20
                    Layout.alignment: Qt.AlignVCenter
                }

                NormalText {
                    text: unacked.length > 0
                        ? ("Alerts (" + unacked.length + ")")
                        : (acked.length > 0 ? "Acknowledged" : "No alerts")
                    font.bold: true
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignVCenter
                }
            }

            ListView {
                id: alertList
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                spacing: Theme.spacingTight
                model: alerts

                delegate: Rectangle {
                    required property var modelData
                    required property int index

                    // Leave room for the overlay scrollbar so it doesn't cover ack/status.
                    width: alertList.width - Theme.marginScrollbar
                    height: rowContent.implicitHeight + Theme.spacingNormal
                    radius: 6
                    color: Theme.categoryBackgroundColor
                    opacity: modelData.acknowledged ? 0.45 : 1.0
                    border.width: 1
                    border.color: "#20ffffff"

                    HoverHandler {
                        id: hoverHandler
                    }

                    Rectangle {
                        anchors.fill: parent
                        radius: parent.radius
                        color: "#18ffffff"
                        visible: hoverHandler.hovered
                    }

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: root.alertActivated(modelData)
                    }

                    RowLayout {
                        id: rowContent
                        anchors.left: parent.left
                        anchors.right: parent.right
                        anchors.top: parent.top
                        anchors.margins: Theme.spacingNormal / 2.0
                        spacing: Theme.spacingNormal

                        ColumnLayout {
                            id: textColumn
                            Layout.fillWidth: true
                            Layout.alignment: Qt.AlignVCenter
                            spacing: 0

                            SmallText {
                                text: modelData.host_id
                                font.bold: true
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            SmallText {
                                text: modelData.category + " · " + modelData.monitor_id
                                color: Theme.textColorDark
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }

                            SmallText {
                                text: modelData.label + (modelData.value !== "" ? (" — " + modelData.value) : "")
                                elide: Text.ElideRight
                                Layout.fillWidth: true
                            }
                        }

                        ColumnLayout {
                            Layout.preferredHeight: textColumn.height
                            Layout.alignment: Qt.AlignRight | Qt.AlignTop
                            spacing: Theme.spacingTight

                            PillText {
                                id: criticalityPill
                                text: modelData.criticality
                                pillColor: Theme.colorForCriticality(modelData.criticality)
                                Layout.alignment: Qt.AlignRight
                            }

                            ImageButton {
                                flatButton: true
                                size: Math.max(
                                    26,
                                    textColumn.height - criticalityPill.height - Theme.spacingTight
                                )
                                imageRelativeWidth: 0.9
                                imageRelativeHeight: 0.9
                                color: Theme.iconColor
                                hoverColor: Theme.highlightColorLight
                                // Show current state: crossed = muted/acked, bell = active.
                                imageSource: modelData.acknowledged
                                    ? "qrc:/main/images/button/alerts-disabled"
                                    : "qrc:/main/images/button/alerts"
                                tooltip: modelData.acknowledged
                                    ? "Clear acknowledgment"
                                    : "Acknowledge alert"
                                Layout.alignment: Qt.AlignRight
                                onClicked: {
                                    LK.acknowledgeMonitorEntry(
                                        modelData.host_id,
                                        modelData.monitor_id,
                                        modelData.entry_id,
                                        !modelData.acknowledged
                                    )
                                }
                            }
                        }
                    }
                }

                ScrollBar.vertical: StyleOverride.ScrollBar {
                    policy: ScrollBar.AsNeeded
                    fadeWhenIdle: false
                    anchors.rightMargin: Theme.spacingNormal
                }
            }
        }
    }

    Shortcut {
        enabled: root.open
        sequence: "Escape"
        onActivated: root.closeRequested()
    }

    onOpenChanged: {
        if (open) {
            panel.forceActiveFocus()
        }
    }
}
