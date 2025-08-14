pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Text"


CategoryGroupBox {
    id: root

    required property string hostId
    property int rowHeight: 45
    property string selectionColor: "transparent"
    property bool isBlocked: !LK.hosts.isHostInitialized(root.hostId)

    label: GroupBoxLabel {
        id: groupBoxLabel
        anchors.left: root.left
        anchors.right: root.right

        text: "Custom commands"
        icon: "qrc:///main/images/button/terminal"
        color: "#252525"
        border.width: 1
        border.color: "#656565"
        showConfigButton: true

        onConfigClicked: {
            root.configClicked()
        }
    }

    signal configClicked()

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
        model: LK.config.getCustomCommands(root.hostId).map(JSON.parse)

        delegate: ItemDelegate {
            id: item
            required property int index
            required property var modelData

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
                        text: item.modelData.name
                    }

                    SmallerText {
                        visible: item.modelData.description.length > 0
                        text: item.modelData.description
                        wrapMode: Text.Wrap
                        width: parent.width
                    }

                    SmallerText {
                        text: "Executes: " + item.modelData.command
                        wrapMode: Text.Wrap
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
                            command_id: "_custom-command",
                            command_params: [item.modelData.command],
                            display_options: {
                                display_icon: "start",
                                display_text: "run",
                                display_style: "button"
                            }
                        }
                        /* TODO?
                        {
                            command_id: "_custom_command",
                            command_params: [],
                            display_options: {
                                display_icon: "stop",
                                display_text: "Stop",
                                display_style: "button"
                            }
                        }
                        */
                    ]

                    Layout.alignment: Qt.AlignHCenter
                    Layout.rightMargin: Theme.spacingNormal

                    onClicked: function(buttonId, commandId, params) {
                        LK.command.executeConfirmed(buttonId, root.hostId, commandId, params)
                    }
                }
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.categoryRefreshMask
        visible: root.isBlocked

        MouseArea {
            anchors.fill: parent
            preventStealing: true
        }
    }

    function refresh() {
        commandList.model = LK.config.getCustomCommands(root.hostId).map(JSON.parse)
    }
}