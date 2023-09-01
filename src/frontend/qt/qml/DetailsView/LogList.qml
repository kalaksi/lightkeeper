import QtQuick 2.15
import QtQuick.Controls 2.15

import "../Text"


ListView {
    id: root 
    required property var rows

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    onHeightChanged: forceLayout()
    spacing: 2
    currentIndex: -1
    clip: true
    focus: true
    highlightFollowsCurrentItem: true
    highlightMoveDuration: 0
    highlight: Rectangle {
        color: Theme.color_highlight_light()
    }

    model: rows.reverse()


    ScrollBar.vertical: ScrollBar {
        id: scrollBar
    }

    delegate: SmallText {
        width: root.width
        text: modelData
        font.family: "monospace"
        wrapMode: Text.WordWrap

        MouseArea {
            anchors.fill: parent
            onClicked: {
                root.currentIndex = index
            }
        }
    }
}