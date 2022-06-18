import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

Item {
    id: table_container
    property alias model: table.model

    // For some reason, ListView delegate doesn't work as expected: doesn't see `value` as defined.
    TableView {
        id: table
        anchors.fill: parent
        onWidthChanged: forceLayout()
        model: lightkeeper_data

        delegate: Rectangle {
            implicitWidth: table_container.width
            implicitHeight: 40
            radius: 8
            color: model.row % 2 == 0 ? "#efefef" : "#e5e5e5"

            RowLayout {
                anchors.fill: parent
                spacing: 20

                RowComponent {
                    HostStatus {
                        id: host_status
                        status: value.status
                    }
                }

                RowComponent {
                    Text {
                        text: value.name
                    }
                }

                RowComponent {
                    PlaceholderText {
                        placeholder: "No FQDN defined"
                        textContent: value.fqdn
                    }
                }

                RowComponent {
                    Layout.fillWidth: true
                    Text {
                        text: value.ip_address
                    }
                }
            }
        }
    }
}