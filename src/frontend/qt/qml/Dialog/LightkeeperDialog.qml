import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import ".."
import "../Text"
import "../Misc"


// This component should be a direct child of main window.
Dialog {
    id: root
    modal: true
    opacity: visible ? 1.0 : 0.0
    title: ""

    property int borderRadius: 6
    property color headerBackground: Theme.titleBarColor

    background: Rectangle {
        width: root.width
        // To hide the top border, add border.width to the height so it goes under header.
        height: root.height - customHeader.height + border.width
        anchors.bottom: parent.bottom
        color: Theme.backgroundColor
        border.color: Theme.borderColor
        border.width: 1
    }

    Overlay.modal: Rectangle {
        color: "#60000000"
    }

    header: Rectangle {
        id: customHeader
        width: root.width
        height: 30
        radius: root.borderRadius
        color: root.headerBackground

        // Cover the rounding on bottom corners.
        Rectangle {
            anchors.bottom: parent.bottom
            width: root.width
            height: root.borderRadius
            color: root.headerBackground
        }

        NormalText {
            anchors.centerIn: parent
            text: root.title
        }
    }

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animationDurationFast
        }
    }
}