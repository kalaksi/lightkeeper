import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl

import Theme


ToolButton {
    id: control

    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display

        icon: control.icon
        text: control.text
        font: control.font
        color: Theme.textColor
    }

    background: ButtonPanel {
        implicitWidth: 20
        implicitHeight: 20

        control: control
        visible: control.down || control.checked || control.highlighted || control.visualFocus || (enabled && control.hovered)
    }
}