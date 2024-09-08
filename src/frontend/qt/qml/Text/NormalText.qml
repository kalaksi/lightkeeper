import QtQuick

BaseText {
    font.pointSize: Theme.fontSize
}

// NOTE: There's currently a bug in Qt where Text is not selectable: https://bugreports.qt.io/browse/QTBUG-14077
// Workaround:
// TextEdit {
//     font.pointSize: 10
//     readOnly: true
//     wrapMode: Text.WordWrap
//     selectByMouse: true
// }