import QtQuick 2.15

// NOTE: There's currently a bug in Qt where Text is not selectable: https://bugreports.qt.io/browse/QTBUG-14077
Text {
    color: Theme.color_text()
    font.pointSize: 10
}

// Workaround:
// TextEdit {
//     color: Material.foreground
//     font.pointSize: 10
//     readOnly: true
//     wrapMode: Text.WordWrap
//     selectByMouse: true
// }