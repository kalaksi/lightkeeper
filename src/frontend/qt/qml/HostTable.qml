import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    id: table_container
    onWidthChanged: table.forceLayout()
    property alias model: table.model

    // For some reason, ListView delegate doesn't work as expected: doesn't see `value` as defined.
    TableView {
        id: table
        anchors.fill: parent
        model: lightkeeper_data

        delegate: Rectangle {
            implicitWidth: parent.width
            implicitHeight: 40
 
            HostStatus {
                id: host_status
                status: value.status
            }

            Text {
                id: host_name
                text: value.name
                anchors.centerIn: parent
            }

            color: model.row % 2 == 0 ? "#efefef" : "#e5e5e5"
        }
    }
}