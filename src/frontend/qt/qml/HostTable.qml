import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

Item {
    id: table_container
    required property var model

    TableView {
        id: table
        anchors.fill: parent
        onWidthChanged: forceLayout()
        model: parent.model
        Component.onCompleted: parent.model.receive_updates()

        delegate: DelegateChooser {

            DelegateChoice {
                column: 0
                delegate: TableItem {
                    firstItem: true
                    implicitWidth: table.width * 0.15

                    HostStatus {
                        implicitHeight: 40
                        id: host_status
                        status: value
                    }
                }
            }
            DelegateChoice {
                column: 1
                delegate: TableItem {
                    implicitWidth: table.width * 0.10

                    Text {
                        text: value
                    }
                }
            }
            DelegateChoice {
                column: 2
                delegate: TableItem {
                    implicitWidth: table.width * 0.20

                    OptionalText {
                        placeholder: "No FQDN defined"
                        text: value
                    }
                }
            }
            DelegateChoice {
                column: 3
                delegate: TableItem {
                    implicitWidth: table.width * 0.20

                    OptionalText {
                        placeholder: "IP address not available"
                        text: value
                    }
                }
            }
            DelegateChoice {
                column: 4
                delegate: TableItem {
                    implicitWidth: table.width * 0.35

                    OptionalText {
                        placeholder: ""
                        text: value
                    }
                }
            }
        }
    }
}