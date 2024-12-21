import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../Text"


CategoryGroupBox {
    id: root

    property int rowHeight: 60
    property string selectionColor: Theme.highlightColorLight

    label: GroupBoxLabel {
        id: groupBoxLabel
        anchors.left: root.left
        anchors.right: root.right

        text: "Custom commands"
        icon: "qrc:///main/images/button/terminal"
        color: "#252525"
        border: 1
        borderColor: "#656565"
        showConfigButton: true
    }

    ListView {
        id: commandList
        anchors.fill: parent
        clip: true
        // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
        boundsBehavior: Flickable.StopAtBounds
        onWidthChanged: forceLayout()
        currentIndex: -1
        highlight: Rectangle {
            color: root.selectionColor
        }
        model: ["command1", "command2", "command3"]

        delegate: ItemDelegate {
            implicitHeight: root.rowHeight
            width: parent.width
            highlighted: ListView.isCurrentItem
            // Background behavior is provided by ListView.
            background: Rectangle {
                color: "transparent"
            }
            onClicked: {
                commandList.currentIndex = commandList.currentIndex === index ? -1 : index
            }

            RowLayout {
                anchors.fill: parent
                anchors.verticalCenter: parent.verticalCenter

                Column {
                    Layout.margins: Theme.spacingNormal
                    Layout.fillWidth: true

                    NormalText {
                        text: modelData
                    }

                    SmallerText {
                        text: modelData
                        wrapMode: Text.WordWrap
                        width: parent.width
                    }
                }

                // Row-level command buttons, aligned to the right.
                CommandButtonRow {
                    id: commandButtonRow

                    size: Math.min(parent.height, 28)
                    collapsible: false
                    commands: [
                        {

                        }

                    ]

                    Layout.alignment: Qt.AlignHCenter

                    onClicked: function(buttonId, commandId, params) {
                        LK.command.execute(buttonId, root.hostId, commandId, params)
                    }
                }
            }
        }
    }
}