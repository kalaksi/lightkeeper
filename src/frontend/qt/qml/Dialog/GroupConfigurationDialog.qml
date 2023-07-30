import QtQuick 2.15
import QtQuick.Controls 1.4
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11

import "../Button"

// This component should be a direct child of main window.
Dialog {
    id: root
    required property var groupSettings

    modal: true
    implicitWidth: 600
    implicitHeight: 650
    standardButtons: Dialog.Ok | Dialog.Cancel

    contentItem: ColumnLayout {

        // Content will overflow behind the buttons with Layout.fillHeight (ugh...), reserve some space with them with this.
        Item {
            Layout.fillWidth: true
            height: 40
        }
    }
}