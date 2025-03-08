import QtQuick

import Theme

Text {
    color: Theme.textColorDark
    font.pixelSize: 8
    font.family: fontLabel.name
    antialiasing: false
    font.letterSpacing: 1

    FontLoader {
        id: fontLabel
        source: "qrc:/main/fonts/pixeloid"
    }
}