import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import "../Button"

Item {
    id: root
    property var commands: []
    property int size: 24
    property int buttonSize: size * 0.95
    property bool flatButtons: false
    property bool roundButtons: true
    property bool collapsible: false
    // Provides a mechanism for forcing the expanded command bar to collapse.
    // Can be used to allow only one command bar to be expanded at a time.
    property bool forceCollapse: false
    property string menuTooltip: "More..."
    property int animationDuration: 150
    property bool hoverEnabled: true

    property var _commandCooldowns: {}
    property bool _showBackground: false
    property bool _showCommands: false
    property var _alwaysShownCommands: commands.filter(command => Theme.allow_collapsing_command(command.command_id) === "0")
    // Shown when `collapsible` is enabled and all of the commands aren't already visible.
    property bool _showMenu: collapsible && _alwaysShownCommands.length < commands.length


    implicitWidth: calculateWidth(!collapsible)
    implicitHeight: size

    signal clicked(string commandId, var params)
    signal expanded()

    Component.onCompleted: {
        root._commandCooldowns = {}

        // No sense in allowing only 1 command to collapse.
        if (root.commands.length < 2) {
            root.collapsible = false
        }
    }

    Rectangle {
        id: background
        anchors.verticalCenter: parent.verticalCenter
        anchors.fill: parent
        anchors.bottomMargin: -1
        anchors.topMargin: -1
        radius: root.size * 0.5
        color: root._showBackground ? Qt.lighter(Theme.categoryBackgroundColor, 1.4) : "transparent"
        border.color: root._showBackground ? Theme.categoryBackgroundColor : "transparent"
        border.width: 1

        Row {
            anchors.verticalCenter: parent.verticalCenter
            anchors.right: parent.right
            spacing: 1

            Repeater {
                id: commandRepeater
                model: !root.collapsible || root._showCommands ?  root.commands : root._alwaysShownCommands

                CommandButton {
                    property string buttonIdentifier: modelData.command_params.length > 0 ?
                        modelData.command_id + '|' + modelData.command_params[0] : modelData.command_id + '|'

                    id: commandButton
                    size: root.buttonSize
                    roundButton: root.roundButtons
                    tooltip: modelData.display_options.display_text
                    imageSource: "qrc:/main/images/button/" + modelData.display_options.display_icon
                    cooldownPercent: 0.0
                    onClicked: root.clicked(modelData.command_id, modelData.command_params)
                    hoverEnabled: root.hoverEnabled
                }
            }

            ImageButton {
                id: menuButton
                visible: root._showMenu
                width: root.buttonSize
                height: root.buttonSize
                flatButton: root.flatButtons
                roundButton: root.roundButtons
                tooltip: root.menuTooltip
                hoverEnabled: root.hoverEnabled
                imageSource: "qrc:/main/images/button/overflow-menu"

                onClicked: !root._showCommands ? root.expand() : root.collapse()
            }
        }
    }

    NumberAnimation {
        id: expandAnimation
        target: root 
        property: "width"
        to: calculateWidth(true)
        duration: root.animationDuration
        easing.type: Easing.OutQuad
        onStopped: {
            root._showCommands = true
        }
    }

    NumberAnimation {
        id: collapseAnimation
        target: root 
        property: "width"
        to: calculateWidth(false)
        duration: root.animationDuration
        easing.type: Easing.OutQuad
        onStopped: {
            root._showBackground = false
        }
    }

    states: [
        State {
            when: root.forceCollapse

            StateChangeScript {
                script: {
                    if (root.forceCollapse) {
                        collapse()
                    }
                }
            }
        }
    ]

    function getButtonIdentifiers() {
        let result = []

        for (let i = 0; i < commandRepeater.count; i++) {
            let button = commandRepeater.itemAt(i)
            result.push(button.buttonIdentifier)
        }

        return result
    }

    function updateCooldown(buttonIdentifier, cooldownPercent) {
        let button = commandRepeater.itemAt(getButtonIdentifiers().indexOf(buttonIdentifier))
        // Assign new value only if necessary.
        if (button !== undefined && button.cooldownPercent !== cooldownPercent) {
            button.cooldownPercent = cooldownPercent
        }
    }

    function calculateWidth(forAllCommands) {
        let spaceForMenu = root._showMenu ? 1 : 0

        if (forAllCommands) {
            return root.size * (commands.length + spaceForMenu)
        }
        else {
            return root.size * (root._alwaysShownCommands.length + spaceForMenu) 
        }
    }

    function expand() {
        if (!root._showCommands) {
            root._showBackground = true
            expandAnimation.start()
            root.expanded()
        }
    }

    function collapse() {
        if (root._showCommands) {
            root._showCommands = false
            collapseAnimation.start()
        }
    }
}