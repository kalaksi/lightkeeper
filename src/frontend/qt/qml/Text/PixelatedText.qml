import QtQuick 2.15
import QtQuick.Controls.Material 2.15

Text {
    color: Qt.darker(Material.foreground, 1.10)
    font.pixelSize: 8
    font.family: fontLabel.name
    antialiasing: false
    font.letterSpacing: 1

    FontLoader {
        id: fontLabel
        source: "qrc:/main/fonts/pixeloid"
    }
}