import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

import PropertyTableModel 1.0

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../js/ValueUnit.js" as ValueUnit


// Table for displaying monitoring data and command buttons.
TableView {
    id: root 
    property string hostId: ""
    property string category: ""
    // MonitoringDatas as QVariants.
    property var monitoring_datas: []
    // CommandDatas as QVariants.
    property var command_datas: []
    property int rowHeight: 24

    // Number of the row that has command row menu expanded.
    // Only one menu can be open at a time.
    property int expandedCommandRow: -1

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    onHeightChanged: forceLayout()
    clip: true
    topMargin: Theme.groupbox_margins()
    bottomMargin: Theme.groupbox_margins()
    implicitHeight: rowHeight * model.rowCount()

    model: PropertyTableModel {
        monitoring_datas: root.monitoring_datas
        command_datas: root.command_datas
    }


    ScrollBar.vertical: ScrollBar {
        id: scrollBar
    }

    Connections {
        target: HostDataManager
        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)
            root.model.end_command_cooldown(commandResult.invocation_id)
        }
    }

    delegate: DelegateChooser {
        id: delegateChooser

        DelegateChoice {
            column: 0
            delegate: Item {
                property string separatorLabel: root.model.get_separator_label(row)
                property bool isSeparator: separatorLabel !== ""

                implicitWidth: root.width * 0.5
                implicitHeight: isSeparator ? root.rowHeight * 2.5 : root.rowHeight

                // Header text for multivalues.
                Label {
                    visible: parent.isSeparator
                    width: root.width
                    height: root.rowHeight * 2
                    padding: 5
                    topPadding: 10
                    horizontalAlignment: Text.AlignHCenter
                    text: parent.separatorLabel

                    background: Rectangle {
                        width: parent.width
                        height: 2
                        anchors.bottom: parent.bottom
                        gradient: Gradient {
                            orientation: Gradient.Horizontal
                            GradientStop { position: 0.0; color: "#404040" }
                            GradientStop { position: 0.5; color: "#555555" }
                            GradientStop { position: 1.0; color: "#404040" }
                        }
                    }
                }

                Label {
                    id: labelComponent
                    visible: !parent.isSeparator
                    anchors.verticalCenter: parent.verticalCenter
                    text: model.value
                }
            }
        }

        DelegateChoice {
            column: 1
            delegate: Item {
                property bool isSeparator: root.model.get_separator_label(row) !== ""
                property var styledValue: JSON.parse(model.value)
                property bool _hasNoChildCommands: CommandHandler.get_child_command_count(root.hostId, root.category) === 0

                visible: !isSeparator
                implicitWidth: _hasNoChildCommands ? root.width * 0.5 : root.width * 0.3
                implicitHeight: root.rowHeight


                // Used to clip overflowing text from the label.
                // Avoiding clip-property on the label itself, since it could cause performance issues.
                // This also allows more customized style for the clipping.
                Rectangle {
                    x: -parent.width * 0.3
                    width: parent.width * 1.3
                    height: parent.height
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.0; color: "#00000000" }
                        GradientStop { position: 0.15; color: Theme.category_background_color() }
                        GradientStop { position: 1.0; color: Theme.category_background_color() }
                    }
                }

                Row {
                    visible: styledValue.display_options.display_style === "ProgressBar"
                    spacing: 5

                    ProgressBar {
                        width: parent.parent.width * 0.75
                        value: parseInt(styledValue.data_point.value, 10) / 100.0
                    }

                    SmallerText {
                        text: ValueUnit.AsText(styledValue.data_point.value, styledValue.display_options.unit)
                        anchors.verticalCenter: parent.verticalCenter
                        lineHeight: 0.9
                    }
                }

                SmallText {
                    visible: styledValue.display_options.display_style === "Text"
                    text: ValueUnit.AsText(styledValue.data_point.value, styledValue.display_options.unit)

                    anchors.verticalCenter: parent.verticalCenter
                    lineHeight: 0.9
                }

                PillText {
                    visible: styledValue.display_options.display_style === "CriticalityLevel"
                    anchors.verticalCenter: parent.verticalCenter
                    text: ValueUnit.AsText(styledValue.data_point.value, styledValue.display_options.unit)
                    pillColor: Theme.pill_color_for_criticality(styledValue.data_point.criticality)
                    height: root.rowHeight * 0.9
                }
            }
        }

        DelegateChoice {
            column: 2
            delegate: Item {
                property bool isSeparator: root.model.get_separator_label(row) !== ""
                property var parsedCommands: JSON.parse(model.value)
                property real _marginRight: scrollBar.width + 8
                property bool _hasNoChildCommands: CommandHandler.get_child_command_count(root.hostId, root.category) === 0

                visible: !isSeparator
                implicitWidth: _hasNoChildCommands ? 1 : root.width * 0.2
                implicitHeight: root.rowHeight

                // Reason for this Rectangle is the same as with delegate 1.
                Rectangle {
                    x: -parent.width * 0.3
                    width: parent.width * 1.35
                    height: parent.height
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.0; color: "#00000000" }
                        GradientStop { position: 0.15; color: Theme.category_background_color() }
                        GradientStop { position: 1.0; color: Theme.category_background_color() }
                    }
                }

                // Row-level command buttons, aligned to the right.
                CommandButtonRow {
                    id: commandButtonRow
                    anchors.right: parent.right
                    // TODO: how to account for scrollbar so margin is not used when scrollbar is not visible?
                    // For scrollbar.
                    anchors.rightMargin: _marginRight
                    visible: parsedCommands.length > 0

                    hostId: root.hostId
                    size: root.rowHeight
                    collapsible: true
                    menuTooltip: "More commands..."
                    commands: parsedCommands
                    forceCollapse: root.expandedCommandRow !== row
                    onExpanded: function() {
                        root.expandedCommandRow = row
                    }

                    onClicked: function(commandId, params) {
                        let invocationId = CommandHandler.execute(root.hostId, commandId, params)
                        let buttonIdentifier = commandId + params.join("")
                        root.model.start_command_cooldown(buttonIdentifier, invocationId)

                        // Does nothing if timer is already running.
                        cooldownTimer.start()
                    }

                    Connections {
                        target: cooldownTimer
                        function onTriggered() {
                            let cooldowns = JSON.parse(root.model.get_command_cooldowns(row))
                            if (Object.keys(cooldowns).length > 0) {
                                commandButtonRow.updateCooldowns(cooldowns)
                            }
                        }
                    }
                }
            }
        }
    }

    // Countdown timer for the cooldown canvas animation
    Timer {
        id: cooldownTimer

        interval: 50
        repeat: true
        running: false
        onTriggered: {
            let cooldownCount = root.model.decrease_command_cooldowns(interval)
            if (cooldownCount === 0) {
                cooldownTimer.stop()
            }
        }
    }
}
