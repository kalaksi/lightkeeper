/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import ".."
import "../Text"

Rectangle {
    id: root

    required property int invocationId
    required property int progress
    required property string statusText
    required property bool cancelling

    signal stopRequested()

    height: 48
    color: Theme.backgroundColor
    border.width: 1
    border.color: Theme.borderColor

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 4
        spacing: Theme.spacingTight

        SmallText {
            visible: root.statusText !== ""
            text: root.statusText
            elide: Text.ElideMiddle
            Layout.fillWidth: true
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingNormal

            ProgressBar {
                id: progressBar

                Layout.fillWidth: true
                Layout.fillHeight: false
                Layout.preferredHeight: 18
                Layout.alignment: Qt.AlignVCenter
                value: root.progress / 100.0

                contentItem: Rectangle {
                    implicitHeight: progressBar.height
                    implicitWidth: progressBar.width
                    color: "#202020"
                    radius: 4

                    Rectangle {
                        height: parent.height
                        width: progressBar.visualPosition * parent.width
                        color: progressBar.palette.highlight
                        radius: parent.radius

                        Behavior on width {
                            NumberAnimation {
                                duration: 200
                                easing.type: Easing.OutQuad
                            }
                        }
                    }
                }
            }

            NormalText {
                lineHeight: 0.9
                text: root.progress + " %"
                Layout.alignment: Qt.AlignVCenter
            }

            ToolButton {
                flat: false
                display: AbstractButton.IconOnly
                icon.source: "qrc:/main/images/button/stop"
                icon.height: 18
                icon.width: 18
                padding: 2
                enabled: !root.cancelling
                Layout.alignment: Qt.AlignVCenter
                onClicked: root.stopRequested()

                ToolTip.visible: hovered
                ToolTip.delay: Theme.tooltipDelay
                ToolTip.text: "Stop transfer"
            }
        }
    }
}
