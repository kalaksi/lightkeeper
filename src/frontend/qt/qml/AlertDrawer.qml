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
import "js/Parse.js" as Parse


Item {
    id: root
    property bool open: false
    property var alerts: []
    property var history: []
    property int historyInvocationId: -1
    // 0 = Alerts, 1 = History
    property int modeIndex: 0

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
        // Stay off-screen when closed; open state docks to the right edge.
        // Use states/transitions (not Behavior) so resize does not animate x.
        x: parent.width
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderLeft: 1
        clip: true
        focus: root.open

        states: State {
            name: "open"
            when: root.open

            PropertyChanges {
                target: panel
                x: root.width - panel.width
            }
        }

        transitions: Transition {
            NumberAnimation {
                property: "x"
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

                Row {
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignVCenter
                    spacing: Theme.spacingTight

                    NormalText {
                        text: root.unacked.length > 0 ? ("Alerts (" + root.unacked.length + ")") : "Alerts"
                        font.bold: root.modeIndex === 0
                        color: root.modeIndex === 0 ? Theme.textColor : Theme.textColorDark

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.modeIndex = 0
                        }
                    }

                    NormalText {
                        text: "|"
                        color: Theme.textColorDark
                    }

                    NormalText {
                        text: "History"
                        font.bold: root.modeIndex === 1
                        color: root.modeIndex === 1 ? Theme.textColor : Theme.textColorDark

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.modeIndex = 1
                        }
                    }
                }
            }

            StackLayout {
                currentIndex: root.modeIndex
                Layout.fillWidth: true
                Layout.fillHeight: true

                ListView {
                    id: alertList
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

                Item {
                    ListView {
                        id: historyList
                        anchors.fill: parent
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds
                        spacing: Theme.spacingTight
                        model: root.history
                        visible: LK.config.showCharts

                        delegate: Rectangle {
                            required property var modelData
                            required property int index

                            width: historyList.width - Theme.marginScrollbar
                            height: historyContent.implicitHeight + Theme.spacingNormal
                            radius: 6
                            color: Theme.categoryBackgroundColor
                            border.width: 1
                            border.color: "#20ffffff"

                            ColumnLayout {
                                id: historyContent
                                anchors.left: parent.left
                                anchors.right: parent.right
                                anchors.top: parent.top
                                anchors.margins: Theme.spacingNormal / 2.0
                                spacing: Theme.spacingTight

                                SmallText {
                                    text: Qt.formatDateTime(new Date(modelData.time * 1000), "yyyy-MM-dd hh:mm:ss")
                                    color: Theme.textColorDark
                                    Layout.fillWidth: true
                                }

                                SmallText {
                                    text: modelData.host_id
                                    font.bold: true
                                    elide: Text.ElideRight
                                    Layout.fillWidth: true
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: Theme.spacingNormal

                                    SmallText {
                                        text: modelData.monitor_id
                                            + (modelData.label !== "" ? (" · " + modelData.label) : "")
                                            + (modelData.value !== "" ? (" — " + modelData.value) : "")
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                        Layout.alignment: Qt.AlignVCenter
                                    }

                                    RowLayout {
                                        spacing: Theme.spacingTight
                                        Layout.alignment: Qt.AlignRight | Qt.AlignVCenter

                                        PillText {
                                            text: modelData.from_level
                                            pillColor: Theme.colorForCriticality(modelData.from_level)
                                            opacity: 0.55
                                        }

                                        OverlayImage {
                                            source: "qrc:/main/images/button/go-next"
                                            color: Theme.iconColor
                                            Layout.preferredWidth: 12
                                            Layout.preferredHeight: 12
                                            Layout.alignment: Qt.AlignVCenter
                                        }

                                        PillText {
                                            text: modelData.to_level
                                            pillColor: Theme.colorForCriticality(modelData.to_level)
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

                    Column {
                        anchors.centerIn: parent
                        spacing: Theme.spacingNormal
                        visible: !LK.config.showCharts
                        width: parent.width * 0.85

                        NormalText {
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            wrapMode: Text.WordWrap
                            text: "Alert history requires the local metrics server."
                        }

                        NormalText {
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            wrapMode: Text.WordWrap
                            color: Theme.textColorDark
                            text: "Enable charts in preferences (restart required)."
                        }
                    }

                    NormalText {
                        anchors.centerIn: parent
                        visible: LK.config.showCharts && root.history.length === 0
                        color: Theme.textColorDark
                        text: "No alert history in the last day"
                    }
                }
            }
        }
    }

    Connections {
        target: LK.metrics

        function onAlertHistoryReceived(invocationId, alertDataJson) {
            if (invocationId !== root.historyInvocationId) {
                return
            }

            root.history = Parse.TryParseJson(alertDataJson) || []
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
            if (root.modeIndex === 1) {
                root.refreshHistory()
            }
        }
    }

    onModeIndexChanged: {
        if (root.open && root.modeIndex === 1) {
            root.refreshHistory()
        }
    }

    function refreshHistory() {
        if (!LK.config.showCharts) {
            root.history = []
            root.historyInvocationId = -1
            return
        }

        root.historyInvocationId = LK.metrics.refreshAlertHistory()
    }
}
