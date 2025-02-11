import QtQuick
import QtQuick.Controls
import QtQuick.Layouts


// Basic ListView with common properties.
Rectangle {
    id: root
    property alias model: customCommandsList.model
    property string modelPropertyName: ""
    property alias currentItem: customCommandsList.currentItem
    property alias currentIndex: customCommandsList.currentIndex

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
            text: root.modelPropertyName !== "" ? modelData[root.modelPropertyName] : modelData
            highlighted: ListView.isCurrentItem
            onClicked: customCommandsList.currentIndex = index
        }
    }
}