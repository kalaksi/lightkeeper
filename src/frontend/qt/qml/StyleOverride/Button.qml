import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl

import Theme

Button {
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
        implicitWidth: 80
        implicitHeight: 24

        control: control
        visible: !control.flat || control.down || control.checked || control.highlighted || control.visualFocus || (enabled && control.hovered)
    }
}