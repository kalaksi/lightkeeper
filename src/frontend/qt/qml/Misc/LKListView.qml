import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    property alias model: customCommandsList.model

    color: Theme.backgroundColor
    border.color: Theme.borderColor
    border.width: 1

    ListView {
        id: customCommandsList
        anchors.fill: parent
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        ScrollBar.vertical: ScrollBar {
            active: true
        }

        delegate: ItemDelegate {
            width: customCommandsList.width
            // TODO
            text: modelData.name
            highlighted: ListView.isCurrentItem
            onClicked: customCommandsList.currentIndex = index
        }
    }
}