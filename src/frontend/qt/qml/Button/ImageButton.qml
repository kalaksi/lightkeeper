/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Theme

import "../Text"
import "../Misc"
import "../StyleOverride"


/// Provides more flexible button with icon and text.
Item {
    id: root
    property string imageSource: ""
    property real imageRelativeWidth: 0.0
    property real imageRelativeHeight: 0.0
    property string color: "transparent"
    property string tooltip: ""
    property string text: ""
    property bool roundButton: false
    property bool flatButton: false
    property bool hoverEnabled: true
    property bool enabled: true
    property bool checkable: false
    property real size: 0.8 * parent.height

    height: root.size
    width: root.size + (buttonText.text !== "" ? buttonText.implicitWidth + Theme.spacingNormal * 3 : 0)

    signal clicked()

    Button {
        flat: root.flatButton
        anchors.fill: parent
        visible: root.roundButton === false
        enabled: root.enabled
        opacity: Theme.opacity(enabled)
        onClicked: root.clicked()
        focusPolicy: Qt.NoFocus
        hoverEnabled: root.hoverEnabled
        checkable: root.checkable

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: Theme.tooltipDelay
        ToolTip.text: root.tooltip

        Row {
            anchors.centerIn: parent
            spacing: Theme.spacingNormal

            OverlayImage {
                id: buttonImage
                source: root.imageSource
                width: root.getIconWidth()
                height: root.getIconHeight()
                color: root.color
            }

            NormalText {
                id: buttonText
                visible: root.text !== ""
                text: root.text
            }
        }
    }

    RoundButton {
        // TODO: For some reason, the hover effect is not working on the RoundButton by default.

        flat: root.flatButton
        anchors.fill: parent
        visible: root.roundButton === true
        enabled: root.enabled
        opacity: root.enabled ? 1.0 : 0.5
        onClicked: root.clicked()
        focusPolicy: Qt.NoFocus
        hoverEnabled: root.hoverEnabled
        checkable: root.checkable

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: Theme.tooltipDelay
        ToolTip.text: root.tooltip

        Row {
            anchors.centerIn: parent
            spacing: Theme.spacingNormal

            OverlayImage {
                id: roundButtonImage
                source: root.imageSource
                color: root.color
                width: root.getIconWidth()
                height: root.getIconHeight()
            }

            NormalText {
                id: roundButtonText
                visible: root.text !== ""
                text: root.text
            }
        }
    }

    /// Icon padding/margins vary a bit so patching a better sizing here.
    function getIconRelativeSize(resourcePath) {
        let icon_name = resourcePath.split("/").pop()
        if (icon_name === "start") {
            return 0.5
        }
        if (icon_name === "stop") {
            return 0.5
        }
        else {
            return 0.8
        }
    }

    function getIconWidth() {
        if (root.imageRelativeWidth > 0.0) {
            return Math.floor(root.imageRelativeWidth * root.height)
        }
        else {
            return Math.floor(getIconRelativeSize(root.imageSource) * root.height)
        }
    }

    function getIconHeight() {
        if (root.imageRelativeHeight > 0.0) {
            return Math.floor(root.imageRelativeHeight * root.height)
        }
        else {
            return Math.floor(getIconRelativeSize(root.imageSource) * root.height)
        }
    }
}