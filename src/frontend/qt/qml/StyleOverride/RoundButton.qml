import QtQuick
import QtQuick.Controls
import org.kde.kirigami as Kirigami

RoundButton {
    Kirigami.Theme.inherit: false
    Kirigami.Theme.backgroundColor: palette.window
    Kirigami.Theme.alternateBackgroundColor: Qt.darker(palette.window, 1.05)
    Kirigami.Theme.hoverColor: palette.highlight
    Kirigami.Theme.focusColor: palette.highlight
}