import QtQuick 2.15

Text {
    color: Theme.color_dark_text()
    font.pixelSize: 8
    font.family: fontLabel.name
    antialiasing: false
    font.letterSpacing: 1

    FontLoader {
        id: fontLabel
        source: "qrc:/main/fonts/pixeloid"
    }
}