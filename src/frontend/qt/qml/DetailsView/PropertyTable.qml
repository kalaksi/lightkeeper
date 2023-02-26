import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

import PropertyTableModel 1.0

import "../Text"


// Table for displaying monitoring data and command buttons.
TableView {
    id: root 
    property var monitoring_datas: []
    property var command_datas: []
    property int rowHeight: 20

    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
    boundsBehavior: Flickable.StopAtBounds
    onWidthChanged: forceLayout()
    // ScrollBar.vertical: ScrollBar { }

    model: PropertyTableModel {
        monitoring_datas: root.monitoring_datas
        command_datas: root.command_datas
    }

    delegate: DelegateChooser {
        id: delegateChooser

        DelegateChoice {
            column: 0
            delegate: Item {
                implicitWidth: root.width * 0.20
                implicitHeight: root.rowHeight

                NormalText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: model.value
                }
            }
        }

        DelegateChoice {
            column: 1
            delegate: Item {
                implicitWidth: root.width * 0.20

                NormalText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: model.value
                }
            }
        }

        DelegateChoice {
            column: 2
            delegate: Item {
                implicitWidth: root.width * 0.20

                NormalText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: model.value
                }
            }
        }

        DelegateChoice {
            column: 3
            delegate: Item {
                implicitWidth: root.width * 0.20

                NormalText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: model.value
                }
            }
        }
    }
}
