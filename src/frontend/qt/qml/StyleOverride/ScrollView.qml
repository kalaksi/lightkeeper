import QtQuick
import QtQuick.Controls.impl
import QtQuick.Controls.Fusion
import QtQuick.Controls.Fusion.impl

import "."

ScrollView {
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

    ScrollBar.vertical: ScrollBar {
        parent: control
        x: control.mirrored ? 0 : control.width - width
        y: control.topPadding
        height: control.availableHeight
        active: control.ScrollBar.horizontal.active
    }

    ScrollBar.horizontal: ScrollBar {
        parent: control
        x: control.leftPadding
        y: control.height - height
        width: control.availableWidth
        active: control.ScrollBar.vertical.active
    }

}