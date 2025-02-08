import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

import "../Button"
import "../Text"


// TODO: remove this
Item {
    id: root

    property var customCommands: LK.command.getCustomCommands(root.hostId).map(JSON.parse)


    Rectangle {
        color: Theme.backgroundColor
        border.color: Theme.borderColor
        border.width: 1

        Layout.fillWidth: true
        Layout.fillHeight: true

        ListView {
            id: commandList
            anchors.fill: parent
            clip: true
            // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
            boundsBehavior: Flickable.StopAtBounds
            onWidthChanged: forceLayout()
            currentIndex: -1
            model: root.customCommands

            delegate: ItemDelegate {
                implicitHeight: root.tableRowHeight
                width: parent.width
                highlighted: ListView.isCurrentItem
                onClicked: commandList.currentIndex = commandList.currentIndex === index ? -1 : index

                Row {
                    anchors.fill: parent
                    anchors.verticalCenter: parent.verticalCenter
                    padding: Theme.spacingNormal

                    Column {
                        width: parent.width * 0.48
                        anchors.verticalCenter: parent.verticalCenter

                        NormalText {
                            // TODO: produces warnings about undefined value when adding new entries.
                            text: modelData.name
                        }

                        SmallerText {
                            text: modelData.description
                            visible: text !== ""
                            wrapMode: Text.WordWrap
                            width: parent.width
                        }
                    }
                }
            }
        }
    }
}