import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

AnimatedSprite {
    property real size: 1.0
    source: "qrc:/main/images/animations/working"
    frameWidth: 22 * size
    frameHeight: 22 * size
    frameCount: 15
    frameDuration: 60
    anchors.centerIn: parent
}