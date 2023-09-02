import QtQuick 2.15

Text {
    id: root
    color: Theme.color_text()
    textFormat: Text.PlainText

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animation_duration()
        }
    }
}