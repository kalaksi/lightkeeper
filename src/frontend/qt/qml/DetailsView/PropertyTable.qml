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
    property var monitoring_datas: []
    property var command_datas: []
    property int rowHeight: 20

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    onHeightChanged: forceLayout()
    clip: true

    ScrollBar.vertical: ScrollBar { }

    model: PropertyTableModel {
        monitoring_datas: root.monitoring_datas
        command_datas: root.command_datas
    }

    implicitHeight: rowHeight * model.rowCount() + 100

    delegate: DelegateChooser {
        id: delegateChooser

        DelegateChoice {
            column: 0
            delegate: Item {
                property string separatorLabel: root.model.get_separator_label(row)
                property bool isSeparator: separatorLabel !== ""

                implicitWidth: root.width * 0.4
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

                visible: !isSeparator
                implicitWidth: root.width * 0.4
                implicitHeight: root.rowHeight

                // Used to clip overflowing text from the label.
                // Avoiding clip-property on the label itself, since it could cause performance issues.
                // This also allows more customized style for the clipping.
                Rectangle {
                    x: -50
                    width: parent.width + 50
                    height: parent.height
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.0; color: "#00000000" }
                        GradientStop { position: 0.2; color: Theme.category_background_color() }
                        GradientStop { position: 1.0; color: Theme.category_background_color() }
                    }
                }

                Row {
                    visible: styledValue.display_options.display_style === "ProgressBar"
                    spacing: 5

                    ProgressBar {
                        width: parent.parent.width * 0.8
                        value: parseInt(styledValue.data_point.value, 10) / 100.0
                    }

                    SmallerText {
                        text: styledValue.data_point.value
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

                visible: !isSeparator
                implicitWidth: root.width * 0.2
                implicitHeight: root.rowHeight

                // Reason for this Rectangle is the same as with delegate 1.
                Rectangle {
                    color: Theme.category_background_color()
                    anchors.fill: parent
                }

                // Row-level command buttons.
                CommandButtonRow {
                    visible: parsedCommands.length > 0
                    collapsed: true
                    menuTooltip: "Commands"
                    commands: parsedCommands
                    onClicked: function(commandId, params) {
                        CommandHandler.execute(root.hostId, commandId, params)
                    }
                }
            }
        }
    }
}