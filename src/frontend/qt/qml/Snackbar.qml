/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "Text"
import "js/Utils.js" as Utils
import "StyleOverride"


Rectangle {
    id: root
    required property string text
    required property string criticality
    property var creationTime: Date.now()
    property int contentPadding: 10
    property int fadeDuration: 200
    property int showDuration: 5000
    property int maximumWidth: 600

    visible: getText() !== ""
    width: Utils.clamp(textContent.implicitWidth + iconBackground.width + root.contentPadding * 3, root.maximumWidth / 3, root.maximumWidth)
    height: 120
    radius: 5
    color: Theme.backgroundColor
    border.width: 1
    border.color: "#50FFFFFF"
    // Alternative way to get some matching color to border:
    // border.color: Qt.darker(Qt.lighter(getColor(), 1.5), 2.0)

    Component.onCompleted: {
        opacity = 0.0
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: scrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    Rectangle {
        id: iconBackground
        anchors.left: parent.left
        anchors.leftMargin: root.border.width
        // "- root.border.width * 2" might be wrong, but otherwise it's not centered.
        width: image.width + root.contentPadding * 2 + iconBackgroundCutoff.width - root.border.width * 2
        height: row.height - root.border.width * 2
        anchors.verticalCenter: parent.verticalCenter
        color: root.getColor()
        radius: 5
    }

    // Cut the rounding on right side
    Rectangle {
        id: iconBackgroundCutoff
        anchors.right: iconBackground.right
        anchors.rightMargin: -root.border.width
        width: parent.radius
        height: iconBackground.height
        anchors.verticalCenter: parent.verticalCenter
        color: root.color
    }

    RowLayout {
        id: row
        spacing: iconBackgroundCutoff.width + root.contentPadding
        anchors.fill: parent

        Image {
            id: image
            antialiasing: true
            source: Theme.iconForCriticality(root.criticality)

            Layout.preferredWidth: 32
            Layout.preferredHeight: 32
            Layout.leftMargin: iconBackground.width / 2 - width / 2
            Layout.alignment: Qt.AlignCenter
        }

        ScrollView {
            id: scrollView
            contentWidth: availableWidth
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: root.contentPadding

            NormalText {
                id: textContent
                width: root.maximumWidth - iconBackground.width - root.contentPadding * 3
                text: root.text
                wrapMode: Text.Wrap
            }
        }
    }

    SequentialAnimation on opacity {
        id: animation

        NumberAnimation {
            to: 0.85
            duration: root.fadeDuration
        }

        PauseAnimation {
            duration: root.showDuration
        }

        NumberAnimation {
            to: 0.0
            duration: root.fadeDuration
        }
    }

    // When hovering, set opacity to 1.0 and stop the animation.
    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        propagateComposedEvents: true
        onEntered: {
            animation.stop()
            root.opacity = 1.0
        }
        onExited: {
            animation.start()
        }
    }

    function getText() {
        if (root.text === "") {
            if (root.criticality === "Error") {
                return "Unknown error occurred"
            } 
            else {
                return ""
            }
        }
        else {
            return root.text
        }
    }

    function getColor() {
        if (root.criticality === "Critical") {
            return "#F25560"
        }
        else if (root.criticality === "Error") {
            return "#FF6065"
        }
        else if (root.criticality === "Warning") {
            return "#FFC734"
        }
        else if (root.criticality === "Info") {
            return Theme.backgroundColorDark
        }
        else if (root.criticality === "Normal") {
            return Theme.backgroundColorDark
        }
        else if (root.criticality === "NoData") {
            return "#FFC734"
        }
    }
}