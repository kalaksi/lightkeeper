import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

TableView {
    id: table

    required property var hostDataManager
    property int rowHeight: 40

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    ScrollBar.vertical: ScrollBar { }

    delegate: DelegateChooser {
        id: delegateChooser

        DelegateChoice {
            column: 0
            delegate: TableCell {
                firstItem: true
                selected: table.model.selected_row === row
                onClicked: table.model.toggle_row(row)
                implicitWidth: table.width * 0.20
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
                onClicked: table.model.toggle_row(row)
                implicitWidth: table.width * 0.10

                NormalText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: value
                }
            }
        }
        DelegateChoice {
            column: 2
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: table.model.toggle_row(row)
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
                onClicked: table.model.toggle_row(row)
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
                onClicked: table.model.toggle_row(row)
                implicitWidth: table.width * 0.3

                MonitorSummary {
                    model: table.hostDataManager.get_monitor_data(value)
                }
            }
        }

        // TODO: some commands here?
        /*
        DelegateChoice {
            column: 5
            delegate: TableCell {
                selected: table.model.selected_row === row
                onClicked: table.model.toggle_row(row)
            }
        }
        */
    }
}
