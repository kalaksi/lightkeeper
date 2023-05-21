import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15


Item {
    id: root
    property int size: 0.8 * parent.height
    property alias tooltip: imageButton.tooltip
    property alias imageSource: imageButton.imageSource
    property alias roundButton: imageButton.roundButton
    property alias hoverEnabled: imageButton.hoverEnabled
    property real cooldownPercent: 0.0

    onCooldownPercentChanged: {
        cooldownCanvas.requestPaint()
    }

    width: root.size
    height: root.size

    signal clicked()

    ImageButton {
        id: imageButton
        anchors.fill: parent
        flatButton: false
        roundButton: true
        onClicked: () => root.clicked()
    }

    // Cooldown animation that blocks the button until timeout timer is finished or finish() called.
    Canvas {
        id: cooldownCanvas
        visible: root.cooldownPercent > 0
        anchors.fill: parent
        onPaint: {
            let context = getContext("2d")
            let radius = width / 2
            context.clearRect(0, 0, width, height)
            context.save()

            context.translate(radius, radius)
            context.rotate(-Math.PI / 2)
            context.beginPath()
            context.arc(0, 0, radius, 0, 2 * Math.PI * (1 - root.cooldownPercent), true)
            context.lineTo(0, 0)
            context.closePath()
            context.fillStyle = "#80FFFFFF"
            context.fill()
            context.restore()
        }
    }

    // Block mouse events from reaching the button.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.AllButtons
        propagateComposedEvents: false
        visible: cooldownCanvas.visible
    }
}