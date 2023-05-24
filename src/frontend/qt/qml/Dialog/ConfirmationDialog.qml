import QtQuick 2.15
import QtQuick.Dialogs 1.1


MessageDialog {
    id: root
    icon: StandardIcon.Question
    title: "Confirmation"
    standardButtons: StandardButton.Yes | StandardButton.No

    Component.onCompleted: visible = true
}