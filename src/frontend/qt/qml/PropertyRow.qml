import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "Text"
import "js/TextTransform.js" as TextTransform

Item {
    id: root

    // NOTE: Beware of issues with required properties and e.g. Repeaters (esp. with nested repeaters will get very confusing!).
    // https://www.qt.io/blog/new-qml-language-features-in-qt-5.15
    // https://stackoverflow.com/questions/62484078/required-property-not-working-with-repeater
    // https://doc.qt.io/qt-5/qtquick-modelviewsdata-modelview.html#models
    required property var modelData // Not really used

    required property string label
    required property string value
    property bool useProgressBar: false
    // Provided only if rowCommands are used.
    property var commandHandler
    property string hostId: ""
    property var commandParams: []
    property var rowCommands: []

    implicitHeight: labelComponent.height
    anchors.left: parent.left
    anchors.right: parent.right


    Rectangle {
        anchors.fill: parent
        radius: 6
        color: Material.background
        visible: hoverArea.containsMouse
    }

    // For highlighting. The whole row needs to be a child so that mouse events are still properly
    // propagated to action buttons too.
    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        propagateComposedEvents: true

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 4
            anchors.rightMargin: 4
            anchors.bottomMargin: 4


            Label {
                id: labelComponent
                text: TextTransform.truncate(root.label, 28)
                verticalAlignment: Text.AlignVCenter
                lineHeight: 0.9
                // TODO: a proper way to center the text better
                bottomPadding: 4

                Layout.fillWidth: true
            }

            ProgressBar {
                value: parseInt(root.value, 10) / 100.0
                visible: root.useProgressBar

                Layout.preferredWidth: 100
            }

            SmallerText {
                text: root.value
                visible: root.useProgressBar
                verticalAlignment: Text.AlignVCenter
                lineHeight: 0.9
                // TODO: a proper way to center the text better
                bottomPadding: 4

                Layout.minimumWidth: 30
            }

            SmallText {
                text: root.value
                visible: !root.useProgressBar
                verticalAlignment: Text.AlignVCenter
                lineHeight: 0.9
                // TODO: a proper way to center the text better
                bottomPadding: 4
            }

            // Row-level command buttons.
            CommandButtonRow {
                id: commands
                collapsed: true
                menuTooltip: "Commands"
                commands: root.rowCommands
                onClicked: function(commandId) {
                    root.commandHandler.execute(root.hostId, commandId, root.commandParams)
                }

                visible: root.rowCommands.length > 0
            }
        }
    }
}