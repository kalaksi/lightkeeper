import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl

ItemDelegate {
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

    background: Rectangle {
        implicitWidth: 100
        implicitHeight: 20
        color: control.down ? Fusion.buttonColor(control.palette, false, true, true)
                            : control.highlighted ? Fusion.highlight(control.palette) : control.palette.base
    }
}
