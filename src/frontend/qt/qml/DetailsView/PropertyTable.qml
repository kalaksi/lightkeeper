import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Styles 1.4

import PropertyTableModel 1.0

import "../StyleOverride"
import ".."
import "../Misc"
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
    property var cooldownTimer: {}

    // Number of the row that has command row menu expanded.
    // Only one menu can be open at a time.
    property int expandedCommandRow: -1
    property int selectedRow: -1

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    onHeightChanged: forceLayout()
    clip: true
    topMargin: Theme.spacingTight
    bottomMargin: Theme.spacingTight

    rowHeightProvider: root.model.get_row_height

    model: PropertyTableModel {
        monitoring_datas: root.monitoring_datas
        command_datas: root.command_datas
        display_options: Theme.get_display_options()
    }


    ScrollBar.vertical: ScrollBar {
        id: scrollBar
    }

    delegate: DelegateChooser {
        id: delegateChooser

        // First delegate is used for labels and descriptions.
        DelegateChoice {
            column: 0
            delegate: PropertyTableCell {
                firstItem: true
                selected: root.selectedRow === row && !isSeparator
                onClicked: toggleRow(row)
                implicitWidth: root.width * root.model.get_column_width(row, column)

                property string separatorLabel: root.model.get_separator_label(row)
                property bool isSeparator: separatorLabel !== ""
                property var labelAndDescription: JSON.parse(model.value)
                // Gradient blocks part of the effective display width so compensate here for scrolling detection.
                property int scrollableWidth: implicitWidth - 20


                // Header text for multivalues.
                Label {
                    visible: parent.isSeparator
                    anchors.bottom: parent.bottom
                    bottomPadding: 8
                    width: root.width
                    
                    horizontalAlignment: Text.AlignHCenter
                    text: parent.separatorLabel

                    background: Rectangle {
                        width: parent.width
                        height: 2
                        anchors.bottom: parent.bottom
                        gradient: Gradient {
                            orientation: Gradient.Horizontal
                            GradientStop { position: 0.0; color: Theme.categoryBackgroundColor }
                            GradientStop { position: 0.5; color: "#606060" }
                            GradientStop { position: 1.0; color: Theme.categoryBackgroundColor }
                        }
                    }
                }

                Column {
                    visible: !parent.isSeparator
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: -3
                    padding: 0
                    leftPadding: parent.radius
                    z: 0

                    ScrollableNormalText {
                        id: labelComponent
                        width: parent.parent.scrollableWidth
                        text: parent.parent.labelAndDescription.label
                        scrollAnimation: root.selectedRow === row
                    }

                    ScrollableSmallerText {
                        visible: parent.parent.labelAndDescription.description !== ""
                        opacity: 0.7
                        width: parent.parent.scrollableWidth
                        text: parent.parent.labelAndDescription.description
                        scrollAnimation: root.selectedRow === row
                    }
                }
            }
        }

        // Second delegate is used for values and tags.
        DelegateChoice {
            column: 1
            delegate: PropertyTableCell {
                gradient: true
                selected: root.selectedRow === row && !isSeparator
                onClicked: toggleRow(row)

                property bool isSeparator: root.model.get_separator_label(row) !== ""
                property var styledValue: JSON.parse(model.value)

                visible: !isSeparator
                implicitWidth: root.width * root.model.get_column_width(row, column)

                Row {
                    width: parent.width
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 2
                    z: 2

                    Row {
                        visible: styledValue.display_options.display_style === "ProgressBar"
                        spacing: 5

                        ProgressBar {
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.parent.width * 0.8
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
                        pillColor: Theme.colorForCriticality(styledValue.data_point.criticality)
                    }

                    Repeater {
                        model: styledValue.data_point.tags

                        PillText {
                            anchors.verticalCenter: parent.verticalCenter
                            text: modelData
                            pillColor: Theme.colorForCriticality("Info")
                        }
                    }
                }
            }
        }

        DelegateChoice {
            column: 2
            delegate: PropertyTableCell {
                selected: root.selectedRow === row && !isSeparator
                onClicked: toggleRow(row)
                lastItem: true

                property bool isSeparator: root.model.get_separator_label(row) !== ""
                property var parsedCommands: JSON.parse(model.value)
                property real _marginRight: scrollBar.width + 8

                visible: !isSeparator
                implicitWidth: root.width * root.model.get_column_width(row, column)

                // Row-level command buttons, aligned to the right.
                CommandButtonRow {
                    id: commandButtonRow
                    visible: parsedCommands.length > 0
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.right: parent.right
                    // TODO: how to account for scrollbar so margin is not used when scrollbar is not visible?
                    // For scrollbar.
                    anchors.rightMargin: _marginRight
                    z: 2

                    size: Math.min(parent.height, 28)
                    collapsible: true
                    menuTooltip: "More commands..."
                    commands: parsedCommands
                    forceCollapse: root.expandedCommandRow !== row
                    onExpanded: {
                        root.expandedCommandRow = row
                    }

                    onClicked: function(commandId, params) {
                        CommandHandler.execute(root.hostId, commandId, params)
                    }

                    Connections {
                        target: root.cooldownTimer

                        function onTriggered() {
                            let buttonIdentifiers = commandButtonRow.getButtonIdentifiers()
                            for (let identifier of buttonIdentifiers) {
                                let cooldownPercent = root.cooldownTimer.getCooldown(identifier)
                                commandButtonRow.updateCooldown(identifier, cooldownPercent)
                            }
                        }
                    }
                }
            }
        }
    }

    function toggleRow(row) {
        if (selectedRow === row) {
            selectedRow = -1
        } else {
            selectedRow = row
        }
    }
}
