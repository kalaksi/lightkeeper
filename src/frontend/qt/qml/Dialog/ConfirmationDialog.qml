import QtQuick
import QtQuick.Dialogs


MessageDialog {
    id: root
    title: "Confirmation"
    buttons: MessageDialog.Ok | MessageDialog.Cancel

    Component.onCompleted: visible = true
}