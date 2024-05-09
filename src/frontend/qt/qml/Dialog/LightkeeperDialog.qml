import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import ".."
import "../Text"
import "../Misc"


// This component should be a direct child of main window.
Dialog {
    id: root
    modal: true
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
}