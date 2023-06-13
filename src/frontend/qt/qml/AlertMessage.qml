import QtQuick 2.15

import "Text"

Item {
    id: root
    required property string text
    required property string criticality
    property real imageScale: 2.0

    implicitWidth: textContent.contentWidth + image.width
    implicitHeight: Math.max(textContent.implicitHeight, image.height)

    Row {
        id: row
        padding: 20
        spacing: 10
        anchors.verticalCenter: parent.verticalCenter

        Image {
            id: image
            antialiasing: true
            source: Theme.icon_for_criticality(root.criticality)
            width: 22 * root.imageScale
            height: 22 * root.imageScale
            anchors.verticalCenter: parent.verticalCenter
        }

        NormalText {
            id: textContent
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            wrapMode: Text.Wrap
            width: root.width - image.width  - row.spacing - row.padding * 2
        }
    }
}
