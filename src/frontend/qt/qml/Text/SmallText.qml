import QtQuick 2.15

Text {
    color: Theme.color_text()
    font.pointSize: 8

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animation_duration()
        }
    }
}