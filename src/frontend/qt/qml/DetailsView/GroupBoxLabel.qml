/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import Qt5Compat.GraphicalEffects

import Theme

import "../Button"
import "../Misc"

Rectangle {
    id: root
    property string text: ""
    property string icon: ""
    property real refreshProgress: 100
    property bool showRefreshButton: false
    property bool showConfigButton: false
    property string accentColor: "#20ffffff"

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight + 10
    layer.enabled: true
    layer.effect: DropShadow {
        color: "#45000000"
        radius: 8
        horizontalOffset: 0
        verticalOffset: 2
    }

    signal refreshClicked()
    signal configClicked()


    Trapezoid {
        // Quick hack to add some detail to some groupboxes.
        visible: root.text === "Network" || root.text === "Storage"
        height: root.height
        width: 50
        color: root.accentColor
    }

    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingNormal

        OverlayImage {
            id: image
            visible: root.icon !== ""
            source: root.icon
            color: label.color
            sourceSize.width: 32
            sourceSize.height: 32
            height: label.implicitHeight * 0.9
            width: height
        }

        Label {
            id: label
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            text: root.cleanupLabel(root.text)
            color: Theme.textColor
            bottomPadding: 4
        }
    }

    Row {
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingTight
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingNormal

        ImageButton {
            visible: root.showConfigButton
            size: 0.8 * root.height
            imageRelativeWidth: 0.8
            imageRelativeHeight: 0.8
            imageSource: "qrc:/main/images/button/configure"
            flatButton: true
            tooltip: "Configure"
            onClicked: root.configClicked()
        }

        RefreshButton {
            visible: root.showRefreshButton
            size: 0.8 * root.height
            onClicked: root.refreshClicked()
            spinning: root.refreshProgress < 100
        }
    }

    function cleanupLabel(text) {
        return text.replace(/-/g, " ")
    }
}