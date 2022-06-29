import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15


RoundButton {
    flat: true
    width: 0.7 * parent.height
    height: 0.7 * parent.height
    anchors.verticalCenter: parent.verticalCenter
    anchors.horizontalCenter: parent.horizontalCenter
    // display: AbstractButton.IconOnly
    // display: AbstractButton.IconOnly

    icon.source: "qrc:/main/images/button/chevron-down"
    icon.height: 0.8 * parent.height
    icon.width: 0.8 * parent.width
    icon.color: "red"
}