import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "./Dialog"


Item {
    id: root
    anchors.fill: parent

    // Modal dialogs
    InputDialog {
        id: inputDialog
    }

    HostConfigurationDialog {
        id: hostConfigDialog
        bottomMargin: 0.13 * parent.height

        onConfigurationChanged: LK.reload()
    }

    CertificateMonitorDialog {
        id: certificateMonitorDialog
        bottomMargin: 0.15 * parent.height
    }

    PreferencesDialog {
        id: preferencesDialog
        bottomMargin: 0.15 * parent.height

        onConfigurationChanged: LK.reload()
    }

    Item {
        id: confirmationDialogsContainer
        anchors.fill: parent

        Repeater {
            id: confirmationDialogRepeater
            model: ListModel {}

            Dialog {
                id: confirmationDialog
                title: "Confirmation"
                modal: true
                visible: true

                property string message: model.message
                property string confirmText: model.confirmText
                property string cancelText: model.cancelText
                property var onConfirm: model.onConfirm

                contentItem: ColumnLayout {
                    spacing: 16

                    Text {
                        text: message
                        wrapMode: Text.WordWrap
                    }

                    RowLayout {
                        spacing: 8
                        anchors.horizontalCenter: parent.horizontalCenter

                        Button {
                            text: confirmText
                            onClicked: {
                                if (onConfirm) {
                                    onConfirm()
                                }
                                confirmationDialog.visible = false
                                confirmationDialogRepeater.model.remove(index)
                            }
                        }

                        Button {
                            text: cancelText
                            onClicked: {
                                confirmationDialog.visible = false
                                confirmationDialogRepeater.model.remove(index)
                            }
                        }
                    }
                }
            }
        }
    }

    function openInput(inputSpecs, onInputValuesGiven) {
        inputDialog.inputSpecs = inputSpecs
        inputDialog.inputValuesGiven.connect(onInputValuesGiven)
        inputDialog.visible = true
    }

    function openHostConfig(hostId) {
        hostConfigDialog.hostId = hostId
        hostConfigDialog.visible = true
    }

    function openCertificateMonitor() {
        certificateMonitorDialog.visible = true
    }

    function openConfirmation(message, confirmText, cancelText, onConfirm) {
        confirmationDialogRepeater.model.append({
            message: message,
            confirmText: confirmText,
            cancelText: cancelText,
            onConfirm: onConfirm
        })
    }
}
