import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15


RoundButton {
    required property string icon_source
    property double scale: 0.5

    flat: true
    width: scale * parent.height
    height: scale * parent.height

    Image {
        source: icon_source
        width: parent.width
        height: parent.height
    }
}