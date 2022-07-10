import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

TableView {
    id: table
    property int rowHeight: 40

    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    Component.onCompleted: model.receive_updates()

    delegate: DelegateChooser {
        id: delegateChooser

        DelegateChoice {
            column: 0
            delegate: TableCell {
                firstItem: true
                selected: table.model.selected_row === row
                onClicked: selectRow(row)
                implicitWidth: table.width * 0.15
                implicitHeight: table.rowHeight

                HostStatus {
                    id: host_status
                    status: value.toLowerCase()
                }
            }
        }
        DelegateChoice {
            column: 1
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: selectRow(row)
                implicitWidth: table.width * 0.10

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: value
                    color: Material.foreground
                }
            }
        }
        DelegateChoice {
            column: 2
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: selectRow(row)
                implicitWidth: table.width * 0.20

                OptionalText {
                    placeholder: "No FQDN defined"
                    text: value
                }
            }
        }
        DelegateChoice {
            column: 3
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: selectRow(row)
                implicitWidth: table.width * 0.20

                OptionalText {
                    placeholder: "IP address not available"
                    text: value
                }
            }
        }
        DelegateChoice {
            column: 4
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: selectRow(row)
                implicitWidth: table.width * 0.3

                MonitorSummary {
                    model: table.model.get_monitor_data(value)
                }
            }
        }

        DelegateChoice {
            column: 5
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: selectRow(row)
            }
        }
    }

    function selectRow(row) {
        if (table.model.selected_row === row) {
            table.model.selected_row = -1
        }
        else {
            table.model.selected_row = row
        }
    }
}
