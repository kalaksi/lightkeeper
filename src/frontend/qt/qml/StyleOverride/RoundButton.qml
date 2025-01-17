import QtQuick 2.15
import QtQuick.Controls 2.15
import org.kde.kirigami 2.19 as Kirigami

RoundButton {
    Kirigami.Theme.inherit: false
    Kirigami.Theme.backgroundColor: palette.window
    Kirigami.Theme.alternateBackgroundColor: Qt.darker(palette.window, 1.05)
    Kirigami.Theme.hoverColor: palette.highlight
    Kirigami.Theme.focusColor: palette.highlight
}