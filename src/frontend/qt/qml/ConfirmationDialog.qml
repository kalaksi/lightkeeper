import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.1

import "js/Parse.js" as Parse

MessageDialog {
    id: root
    icon: StandardIcon.Question
    standardButtons: Dialog.Yes | Dialog.No
    visible: true
    text: "TODO"
}