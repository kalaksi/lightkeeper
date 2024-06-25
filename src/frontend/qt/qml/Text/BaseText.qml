import QtQuick 2.15

Text {
    id: root
    color: Theme.textColor
    textFormat: Text.PlainText

    Behavior on opacity {
        NumberAnimation {
            duration: Theme.animationDurationFast
        }
    }
}