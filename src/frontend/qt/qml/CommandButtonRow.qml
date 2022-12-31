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
    property int animationDuration: 175
    property bool _showCommands: false

    implicitWidth: calculateWidth(!collapsed)
    implicitHeight: size

    signal clicked(string commandId)

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
                model: !root.collapsed || root._showCommands ? root.commands : []

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
                visible: root.collapsed

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
        if (forAllCommands === true) {
            return root.size * (commands.length + 1)
        }
        else {
            return root.size
        }
    }
}