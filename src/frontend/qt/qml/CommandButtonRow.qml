import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15


Item {
    id: root
    property var commands: []
    property int size: 24
    property int buttonSize: size * 0.95
    property bool flatButtons: true
    property bool roundButtons: true
    property bool collapsed: false
    property string menuTooltip: "More..."
    property int animationDuration: 150
    property bool _showCommands: false
    property var _alwaysShownCommands: commands.filter(command => !command.display_options.collapsible)
    // Shown when `collapsed` is enabled and all of the commands aren't already visible.
    property bool _showMenu: collapsed && _alwaysShownCommands.length < commands.length


    implicitWidth: calculateWidth(!collapsed)
    implicitHeight: size

    signal clicked(string commandId)

    Component.onCompleted: {
        // No sense in allowing only 1 command to collapse.
        if (root.commands.length < 2) {
            root.collapsed = false
        }
    }

    Rectangle {
        id: background
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        radius: root.size * 0.5
        color: root.collapsed ? Qt.lighter(Material.background, 1.25) : "transparent"
        width: calculateWidth(!root.collapsed)

        Row {
            anchors.right: parent.right

            Repeater {
                model: !root.collapsed || root._showCommands ?  root.commands : root._alwaysShownCommands
                ImageButton {
                    width: root.buttonSize
                    height: root.buttonSize
                    flatButton: root.flatButtons
                    roundButton: root.roundButtons
                    tooltip: modelData.display_options.display_text
                    imageSource: "qrc:/main/images/button/" + modelData.display_options.display_icon
                    onClicked: root.clicked(modelData.command_id)
                }
            }

            ImageButton {
                id: menuButton
                width: root.buttonSize
                height: root.buttonSize
                flatButton: root.flatButtons
                roundButton: root.roundButtons
                tooltip: root.menuTooltip
                imageSource: "qrc:/main/images/button/overflow-menu"
                visible: root._showMenu

                onClicked: {
                    if (root._showCommands) {
                        root._showCommands = false
                        collapseAnimation.start()
                    }
                    else {
                        expandAnimation.start()
                        root._showCommands = true
                    }
                }
            }
        }
    }

    NumberAnimation {
        id: expandAnimation
        target: background
        property: "width"
        to: calculateWidth(true)
        duration: root.animationDuration
        easing.type: Easing.OutQuad
    }

    NumberAnimation {
        id: collapseAnimation
        target: background
        property: "width"
        to: calculateWidth(false)
        duration: root.animationDuration
        easing.type: Easing.OutQuad
    }

    function calculateWidth(forAllCommands) {
        let spaceForMenu = root._showMenu ? 1 : 0

        if (forAllCommands === true) {
            return root.size * (commands.length + spaceForMenu)
        }
        else {
            return root.size * (root._alwaysShownCommands.length + spaceForMenu) 
        }
    }
}