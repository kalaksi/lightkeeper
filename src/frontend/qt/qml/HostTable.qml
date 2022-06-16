import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    id: table_container
    onWidthChanged: table.forceLayout()
    property alias model: table.model

    TableView {
        id: table
        columnSpacing: 1
        rowSpacing: 1
        anchors.fill: parent

        TableModelColumn {
            display: "Status"
        }

        TableModelColumn {
            display: "Name"
        }

        TableModelColumn {
            display: "FQDN"
        }

        TableModelColumn {
            display: "IP"
        }

        delegate: Rectangle {
            implicitWidth: table_container.width / table.model.columnCount()
            implicitHeight: 40
 
            Text {
                text: value
                anchors.centerIn: parent
            }

            color: "#efefef"
        }
    }
}