import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import Qt5Compat.GraphicalEffects

import "../Button"

Rectangle {
    id: root
    property string text: ""
    property string icon: ""
    property real refreshProgress: 100

    color: "#00000000"
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

    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        spacing: Theme.spacingNormal

        Image {
            visible: root.icon !== ""
            source: root.icon
            sourceSize.width: 32
            sourceSize.height: 32
            height: label.implicitHeight * 0.9
            width: height

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: label.color
            }
        }

        Label {
            id: label
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            text: cleanupLabel(root.text)
            color: Theme.textColor
            bottomPadding: 4
        }
    }

    RefreshButton {
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: Theme.spacingTight
        onClicked: root.refreshClicked()
        spinning: root.refreshProgress < 100
    }

    function cleanupLabel(text) {
        return text.replace(/-/g, " ")
    }
}