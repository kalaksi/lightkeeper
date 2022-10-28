import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property var selections: []
    property var text: ""
    property var errorText: ""
    property var criticality: ""

    Rectangle {
        color: Material.background
        anchors.fill: parent
    }

    Row {
        id: searchBox
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: 20

        spacing: 30

        ComboBox {
            id: selection
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width * 0.20
            model: root.selections
        }

        CheckBox {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width * 0.10
            checked: true
            text: "Regex search"
        }

        TextField {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width * 0.45
            placeholderText: "Search..."
            color: Material.foreground
            // width: root.width * 0.5
        }

        Button {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width * 0.10
            text: "Search"
        }
    }

    ScrollView {
        anchors.top: searchBox.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 5

        ScrollBar.vertical.position: contentHeight
        ScrollBar.vertical.policy: ScrollBar.AlwaysOn
        // ScrollBar.vertical.position: 0.0

        TextArea {
            anchors.fill: parent
            readOnly: true
            activeFocusOnPress: false
            text: root.text
            font.family: "monospace"

            cursorPosition: length - 1
        }
    }
}