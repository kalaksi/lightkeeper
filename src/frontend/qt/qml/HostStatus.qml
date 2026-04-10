/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Layouts

import Lightkeeper 1.0

import "Text"
import "Misc"

Item {
    id: root
    required property string status
    property var colors: ({})
    property var secondaryColors: ({})
    property bool showIcon: true
    anchors.fill: parent

    Component.onCompleted: {
        colors = {
            // Color in theme model was not good.
            up: "forestgreen",
            down: Theme.colorForCriticality("Error"),
            pending: "orange",
            unknown: Theme.colorForCriticality("NoData"),
            _: "orange",
        }
        secondaryColors = {
            up: "forestgreen",
            down: Theme.colorForCriticality("Error"),
            pending: "orange",
            unknown: "orange",
            _: "orange",
        }
    }

    FontLoader {
        id: fontStatus
        source: "qrc:/main/fonts/pressstart2p"
    }

    RowLayout {
        anchors.fill: parent

        OverlayImage {
            id: image
            antialiasing: true
            source: "qrc:/main/images/status/" + root.status
            color: root.getSecondaryColor(root.status)
            visible: root.showIcon

            Layout.leftMargin: root.showIcon ? 0.4 * parent.height : 0
            Layout.rightMargin: root.showIcon ? 0.4 * parent.height : 0
            Layout.preferredWidth: root.showIcon ? 0.7 * parent.height : 0
            Layout.preferredHeight: root.showIcon ? 0.7 * parent.height : 0
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
        }

        NormalText {
            text: root.status.toUpperCase()
            font.family: fontStatus.name
            color: root.getColor(root.status)
            // style: root.status === "unknown" ? Text.Outline : Text.Normal
            styleColor: root.status === "unknown" ? root.getColor("pending") : "transparent"

            Layout.fillWidth: true
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
        }
    }

    function getColor(status) {
        let result = root.colors[status]
        if (typeof result === "undefined") {
            return root.colors["_"]
        }
        return result
    }

    function getSecondaryColor(status) {
        let result = root.secondaryColors[status]
        if (typeof result === "undefined") {
            return root.secondaryColors["_"]
        }
        return result
    }
}