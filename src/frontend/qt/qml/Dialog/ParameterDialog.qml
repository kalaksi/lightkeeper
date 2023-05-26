import QtQuick 2.0
import QtQuick.Controls 2.15

Item {
    id: root
    property string title: "Dialog"
    property bool useBackgroundOverlay: true
    property var labels: []

    // This component should be a direct child of main window.
    anchors.fill: parent

    Rectangle {
        id: backgroundOverlay
        anchors.fill: parent
        color: "black"
        opacity: 0.5
        visible: useBackgroundOverlay

        // Prevent clicks from passing through.
        MouseArea {
            anchors.fill: parent
            onClicked: {
                event.accepted = true;
            }
        }
    }

    Dialog {
        id: dialog
        title: root.title
        width: 400
        height: 300
        standardButtons: Dialog.Ok | Dialog.Cancel

        contentItem: Column {
            spacing: 10
            Repeater {
                model: root.labels

                Item {
                    anchors.fill: parent

                    Label {
                        text: modelData
                    }
                    TextField {
                        placeholderText: "Enter a number"
                        validator: DoubleValidator {}
                    }
                }
            }
        }

        onAccepted: {
            let values = []
            for (let i = 0; i < root.labels.length; i++) {
                values.push(dialog.contentItem.children[i].children[1].text)
            }
            console.log(values)
        }
    }
}