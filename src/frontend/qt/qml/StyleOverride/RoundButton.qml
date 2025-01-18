import QtQuick 2.15
import QtQuick.Controls 2.15
import org.kde.kirigami 2.19 as Kirigami

RoundButton {
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

    Kirigami.Theme.inherit: false
    Kirigami.Theme.backgroundColor: palette.window
    Kirigami.Theme.alternateBackgroundColor: Qt.darker(palette.window, 1.05)
    Kirigami.Theme.hoverColor: palette.highlight
    Kirigami.Theme.focusColor: palette.highlight
}