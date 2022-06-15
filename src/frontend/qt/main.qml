import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0

Window {
    width: 400
    height: 400
    visible: true

    TableView {
        anchors.fill: parent
        columnSpacing: 1
        rowSpacing: 1
        model: lightkeeper_data
        delegate: Rectangle {
            implicitWidth: 100
            implicitHeight: 50
            Text {
                text: value
            }

            color: "#efefef"
        }
    }
}