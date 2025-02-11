import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../Text"


// Basic ListView with common properties.
Rectangle {
    id: root
    property alias model: originalListView.model
    property string labelPropertyName: ""
    property string descriptionPropertyName: ""
    property alias currentItem: originalListView.currentItem
    property alias currentIndex: originalListView.currentIndex

    color: Theme.backgroundColor
    border.color: Theme.borderColor
    border.width: 1

    ListView {
        id: originalListView 
        anchors.fill: parent
        clip: true
        boundsBehavior: Flickable.StopAtBounds

        ScrollBar.vertical: ScrollBar {
            active: true
        }

        delegate: ItemDelegate {
            width: originalListView.width
            implicitHeight: labelText.height + descriptionText.height + Theme.spacingNormal
            highlighted: ListView.isCurrentItem
            onClicked: originalListView.currentIndex = index

            Column {
                anchors.verticalCenter: parent.verticalCenter
                leftPadding: Theme.spacingNormal
                spacing: -3

                NormalText {
                    id: labelText
                    width: parent.parent.scrollableWidth
                    text: root.labelPropertyName !== "" ? modelData[root.labelPropertyName] : modelData
                }

                SmallerText {
                    id: descriptionText
                    visible: root.descriptionPropertyName !== ""
                    opacity: 0.7
                    text: modelData[root.descriptionPropertyName] !== undefined ? modelData[root.descriptionPropertyName] : ""
                }
            }
        }
    }
}