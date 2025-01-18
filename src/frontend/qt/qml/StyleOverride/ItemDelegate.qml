import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl

ItemDelegate {
    id: control

    background: Rectangle {
        implicitWidth: 100
        implicitHeight: 20
        color: control.down ? Fusion.buttonColor(control.palette, false, true, true)
                            : control.highlighted ? Fusion.highlight(control.palette) : Theme.baseColor
    }
}
