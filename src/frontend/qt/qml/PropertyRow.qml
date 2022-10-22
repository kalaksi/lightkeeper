import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

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
    property string targetId: ""
    property var rowCommands: []

    implicitHeight: labelComponent.height
    anchors.left: parent.left
    anchors.right: parent.right

    RowLayout {
        anchors.fill: parent

        Label {
            id: labelComponent
            text: TextTransform.truncate(root.label, 28)

            Layout.fillWidth: true
        }

        ProgressBar {
            value: parseInt(root.value, 10) / 100.0
            visible: root.useProgressBar

            Layout.preferredWidth: 100
        }

        NormalText {
            text: root.value
            visible: root.useProgressBar
            font.pixelSize: 8

            Layout.minimumWidth: 30
        }

        NormalText {
            text: root.value
            visible: !root.useProgressBar
        }

        // Row-level command buttons.
        CommandButtonRow {
            id: commands

            commands: root.rowCommands
            onClicked: function(commandId) {
                root.commandHandler.execute(root.hostId, commandId, root.targetId)
            }

            visible: root.rowCommands.length > 0
        }
    }
}