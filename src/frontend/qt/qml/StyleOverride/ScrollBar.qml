/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls.Fusion

ScrollBar {
    id: control

    property bool fadeWhenIdle: true

    minimumSize: {
        let track = control.orientation === Qt.Vertical ? control.height : control.width
        return track > 0 ? Math.min(1, 50 / track) : 0
    }

    contentItem: Rectangle {
        implicitWidth: control.interactive ? 8 : 3
        implicitHeight: control.interactive ? 8 : 3

        radius: width / 1.5
        color: control.pressed ? "#3a4045" : "#474d54"
        opacity: 0.0

        states: State {
            name: "active"
            when: control.policy === ScrollBar.AlwaysOn
                || (control.size < 1.0 && (control.active || !control.fadeWhenIdle))
            PropertyChanges { control.contentItem.opacity: 0.75 }
        }

        transitions: Transition {
            from: "active"
            SequentialAnimation {
                PauseAnimation { duration: 450 }
                NumberAnimation { target: control.contentItem; duration: 200; property: "opacity"; to: 0.0 }
            }
        }
    }
}