/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Theme

import "../Misc"


Item {
    id: root
    property bool selected: false
    property bool gradient: false
    property bool firstItem: false
    property bool lastItem: false
    property int radius: 9


    signal clicked()

    // Background for semicircle.
    Rectangle {
        visible: root.firstItem && root.selected
        height: leftRounding.height
        width: leftRounding.width / 2
        color: root.getBackgroundColor(false)
        z: 11
    }

    SemiCircle {
        id: leftRounding
        visible: root.firstItem && root.selected
        radius: root.radius
        color: root.getBackgroundColor(root.selected)
        // Force on top so that scrolling text is cut neatly.
        z: 12
    }

    Rectangle {
        id: background
        visible: root.gradient === false
        anchors.fill: parent
        color: root.getBackgroundColor(root.selected)

        MouseArea {
            anchors.fill: parent
            onClicked: root.clicked()
        }
    }

    // Backgrounds are also used to clip overflowing text from label on the left.
    // Avoids clip-property on the label itself, since it could cause performance issues if not used sparingly.
    // This also allows more customized style for the clipping.
    Rectangle {
        id: gradientBackground
        visible: root.gradient === true
        x: -parent.width * 0.3
        width: parent.width * 1.3
        height: parent.height
        radius: root.lastItem ? root.radius : 0
        // Sometimes drawing in order doesn't work correctly for some reason and this Rectangle doesn't cover the content like it should.
        // Therefore, we need to play around with z indices a bit in PropertyTable and here.
        z: 1

        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: "#00000000" }
            GradientStop { position: 0.15; color: root.getBackgroundColor(root.selected) }
            GradientStop { position: 1.0; color: root.getBackgroundColor(root.selected) }
        }
    }



    function getBackgroundColor(selected) {
        if (selected === true) {
            return Qt.darker(Theme.categoryBackgroundColor, 1.20)
        }
        else {
            return Theme.categoryBackgroundColor
        }
    }
}