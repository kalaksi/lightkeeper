/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: AGPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls

import Lightkeeper 1.0

import "../js/TextTransform.js" as TextTransform


GroupBox {
    id: root

    default property alias content: contentItem.data
    property string categoryName: ""
    property string hostId: ""
    property alias refreshProgress: groupBoxLabel.refreshProgress
    property bool isBlocked: refreshProgress < 100
    property bool showAutoExpandButton: true
    property bool autoExpandEnabled: {
        let _ = LK.config.autoExpandCategories
        return LK.config.isAutoExpandCategory(root.categoryName)
    }

    leftPadding: Theme.spacingTight
    rightPadding: Theme.spacingTight

    background: Rectangle {
        color: Theme.categoryBackgroundColor
    }


    // Custom label provides more flexibility.
    label: GroupBoxLabel {
        id: groupBoxLabel
        width: root.width

        text: TextTransform.capitalize(root.categoryName)
        icon: Theme.categoryIcon(root.categoryName)
        color: Theme.categoryColor(root.categoryName)

        showRefreshButton: true
        showConfigButton: true
        showAutoExpandButton: root.showAutoExpandButton
        autoExpandEnabled: root.autoExpandEnabled
        onRefreshClicked: root.refreshClicked()
        onConfigClicked: root.configClicked()
        onAutoExpandToggled: LK.config.setAutoExpandCategory(root.categoryName, !root.autoExpandEnabled)
    }

    signal refreshClicked()
    signal configClicked()


    // Child components get put here.
    Item {
        id: contentItem
        anchors.fill: parent
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.categoryRefreshMask
        visible: root.isBlocked
        z: 10

        MouseArea {
            anchors.fill: parent
            preventStealing: true
        }
    }
}
