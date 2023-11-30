import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Window 2.15

import ".."


Window {
    property var identifier: ""
    property var text: ""
    property var errorText: ""
    property var criticality: ""

    id: root
    visible: true
    color: Theme.backgroundColor

    Dialog {
        modal: false
        standardButtons: Dialog.Ok
        implicitHeight: root.height
        implicitWidth: root.width
        Component.onCompleted: visible = true

        onAccepted: root.close()

        WorkingSprite {
            visible: root.text === "" && root.errorText === ""
        }

        ScrollView {
            visible: root.text !== ""
            anchors.fill: parent

            JsonTextFormat {
                jsonText: root.text
            }
        }

        AlertMessage {
            id: textContent
            text: root.errorText
            criticality: root.criticality
            visible: root.errorText !== ""
        }
    }
}