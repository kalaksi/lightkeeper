import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import "Text"

TableView {
    id: root 
    property int rowHeight: 40
    property var _monitorHighlights: {}
    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()

    Component.onCompleted: {
        root._monitorHighlights = {}
    }

    Connections {
        target: root.model

        function onSelectedRowChanged() {
            centerRow()
        }
    }

    ScrollBar.vertical: ScrollBar {
        id: scrollBar
        policy:ScrollBar.AlwaysOn
    }

    delegate: DelegateChooser {
        id: delegateChooser

        DelegateChoice {
            column: 0
            delegate: TableCell {
                firstItem: true
                selected: root.model.selectedRow === row
                onClicked: root.model.toggleRow(row)
                implicitWidth: root.width * 0.15
                implicitHeight: root.rowHeight

                HostStatus {
                    id: host_status
                    status: value.toLowerCase()
                }
            }
        }
        DelegateChoice {
            column: 1
            delegate: TableCell {
                selected: root.model.selectedRow === row
                onClicked: root.model.toggleRow(row)
                implicitWidth: root.width * 0.15

                NormalText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: value
                }
            }
        }
        DelegateChoice {
            column: 2
            delegate: TableCell {
                selected: root.model.selectedRow === row
                onClicked: root.model.toggleRow(row)
                implicitWidth: root.width * 0.25

                OptionalText {
                    anchors.verticalCenter: parent.verticalCenter
                    placeholder: "No FQDN defined"
                    text: value
                }
            }
        }
        DelegateChoice {
            column: 3
            delegate: TableCell {
                selected: root.model.selectedRow === row
                onClicked: root.model.toggleRow(row)
                implicitWidth: root.width * 0.15

                OptionalText {
                    anchors.verticalCenter: parent.verticalCenter
                    placeholder: "IP address not available"
                    text: value
                }
            }
        }
        DelegateChoice {
            column: 4
            delegate: TableCell {
                selected: root.model.selectedRow === row
                onClicked: root.model.toggleRow(row)
                implicitWidth: root.width * 0.3

                MonitorSummary {
                    model: HostDataManager.get_summary_monitor_data(value)
                    highlights: value in root._monitorHighlights ? root._monitorHighlights[value] : {}
                }
            }
        }
    }

    function highlightMonitor(hostId, monitorId, newCriticality) {
        if (!(hostId in root._monitorHighlights)) {
            root._monitorHighlights[hostId] = {}
        }

        root._monitorHighlights[hostId][monitorId] = newCriticality
    }

    function centerRow() {
        let rowsBeforeSelected = Math.min(root.model.selectedRow, 2)
        root.contentY = root.model.selectedRow * root.rowHeight - rowsBeforeSelected * root.rowHeight
    }
}
