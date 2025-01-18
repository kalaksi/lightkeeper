import QtQuick
import QtQuick.Controls
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl

Button {
    id: control

    palette.alternateBase: Theme.alternateBaseColor
    palette.base: Theme.baseColor
    palette.brightText: "#ffffff"
    palette.button: "#31363b"
    palette.buttonText: Theme.textColor
    palette.dark: "#441618"
    palette.highlight: "#3daee9"
    palette.highlightedText: "#fcfcfc"
    palette.light: "#474d54"
    palette.link: "#1d99f3"
    palette.linkVisited: "#9b59b6"
    palette.mid: "#24282b"
    palette.midlight: "#3a4045"
    palette.shadow: "#0f1012"
    palette.text: "#fcfcfc"
    palette.toolTipBase: "#31363b"
    palette.toolTipText: "#fcfcfc"
    palette.window: "#2a2e32"
    palette.windowText: "#fcfcfc"

    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display

        icon: control.icon
        text: control.text
        font: control.font
        color: control.palette.buttonText
    }

    background: ButtonPanel {
        implicitWidth: 80
        implicitHeight: 24

        control: control
        visible: !control.flat || control.down || control.checked || control.highlighted || control.visualFocus
            || (enabled && control.hovered)
    }
}