import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl


TextField {
    id: control

    // This is actually the original implementation but defining it again makes the palette work.
    background: Rectangle {
        implicitWidth: 120
        implicitHeight: 24

        radius: 2
        color: palette.base
        border.color: control.activeFocus ? Fusion.highlightedOutline(control.palette) : Fusion.outline(control.palette)

        Rectangle {
            x: 1; y: 1
            width: parent.width - 2
            height: parent.height - 2
            color: "transparent"
            border.color: Color.transparent(Fusion.highlightedOutline(control.palette), 40 / 255)
            visible: control.activeFocus
            radius: 1.7
        }

        Rectangle {
            x: 2
            y: 1
            width: parent.width - 4
            height: 1
            color: Fusion.topShadow
        }
    }
}