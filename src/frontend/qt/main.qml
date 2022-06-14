import QtQuick 2.15
import QtQuick.Window 2.15
import Qt.labs.qmlmodels 1.0

Window {
    width: 400
    height: 400
    visible: true

    ListView {
        anchors.fill: parent;
        model: _model;
        delegate: Text {
            text: value.name
        }
    }
}