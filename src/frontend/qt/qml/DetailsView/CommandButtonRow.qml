pragma ComponentBehavior: Bound
import QtQuick

import Theme

import "../Button"

Item {
    id: root
    /// List of CommandButtonData objects.
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

    property bool _showBackground: false
    property bool _showCommands: false
    property var _alwaysShownCommandIds: commands.filter(command => Theme.allowCollapsingCommand(command.command_id) === "0")
                                                 .map(command => command.command_id)
    // Shown when `collapsible` is enabled and all of the commands aren't already visible.
    property bool _showMenu: collapsible && _alwaysShownCommandIds.length < commands.length
    /// Have to store button states so that they can be restored when expanding/collapsing.
    property var _buttonProgressStates: {}


    implicitWidth: calculateWidth(!collapsible)
    implicitHeight: size

    signal clicked(string buttonId, string commandId, var params)
    signal expanded()

    Component.onCompleted: {
        // No sense in allowing only 1 command to collapse.
        if (root.commands.length < 2) {
            root.collapsible = false
        }

        root._buttonProgressStates = {}
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
                model: root.commands
                
                CommandButton {
                    required property var modelData

                    visible: !root.collapsible || root._showCommands || root._alwaysShownCommandIds.includes(modelData.command_id)
                    buttonId: root.createButtonId(modelData)
                    size: root.buttonSize
                    roundButton: root.roundButtons
                    tooltip: modelData.display_options.display_text
                    imageSource: "qrc:/main/images/button/" + modelData.display_options.display_icon
                    progressPercent: root._buttonProgressStates[buttonId] !== undefined ? root._buttonProgressStates[buttonId] : 100
                    onClicked: {
                        root.clicked(buttonId, modelData.command_id, modelData.command_params)
                    }
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
        to: root.calculateWidth(true)
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
        to: root.calculateWidth(false)
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
                        root.collapse()
                    }
                }
            }
        }
    ]

    function createButtonId(commandButton) {
        if (commandButton.command_params.length > 0) {
            return commandButton.command_id + '|' + commandButton.command_params[0]

        }
        else {
            return commandButton.command_id + '|'
        }
    }

    function getButtonIdentifiers() {
        let result = []
        for (let i = 0; i < commandRepeater.count; i++) {
            let button = commandRepeater.itemAt(i) as CommandButton
            result.push(button.buttonId)
        }
        return result
    }

    // Allows updating one button at a time.
    // State has to be stored and handled on higher level and not in e.g. CommandButton or CommandButtonRow since those are not persistent.
    function updateProgress(buttonId, progressPercent) {
        let button = commandRepeater.itemAt(getButtonIdentifiers().indexOf(buttonId)) as CommandButton

        // Assign new value only if necessary.
        if (button !== null && button.progressPercent !== progressPercent) {
            // console.log("Updating progress for button " + buttonId + " to " + progressPercent)
            root._buttonProgressStates[buttonId] = progressPercent
            button.progressPercent = progressPercent
        }
    }

    function calculateWidth(forAllCommands) {
        let spaceForMenu = root._showMenu ? 1 : 0

        if (forAllCommands) {
            return root.size * (commands.length + spaceForMenu)
        }
        else {
            return root.size * (root._alwaysShownCommandIds.length + spaceForMenu) 
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