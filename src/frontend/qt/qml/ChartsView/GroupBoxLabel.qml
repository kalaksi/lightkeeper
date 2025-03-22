import QtQuick
import QtQuick.Controls

import Theme

import "../Misc"

Rectangle {
    id: root
    property string text: ""
    property string icon: ""
    property bool showRefreshButton: false
    property bool showConfigButton: false
    property string accentColor: "#20ffffff"

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight + 4

    signal refreshClicked()
    signal configClicked()

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
            bottomPadding: 2
        }
    }

    function cleanupLabel(text) {
        return text.replace(/-/g, " ")
    }
}