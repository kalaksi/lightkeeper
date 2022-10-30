import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

Item {
    // TODO: loading indicator and indicator (translucent text maybe) for empty results 
    // TODO: indicator for no search results
    id: root
    required property var commandHandler
    property string hostId: ""
    property var selections: []
    property string text: ""
    property string errorText: ""
    property string criticality: ""
    property string _unitId: ""


    Rectangle {
        color: Material.background
        anchors.fill: parent
    }

    Row {
        // TODO: Separate component: SearchableTextArea
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

            // TODO: don't hard-code command id.
            onActivated: function(index) {
                root._unitId = root.selections[index].toLowerCase()
                // TODO: use invocation id?
                root.commandHandler.execute(root.hostId, "logs", [root._unitId])
            }
        }

        CheckBox {
            anchors.leftMargin: 30
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width * 0.10
            checked: true
            text: "Regex search"
        }

        Row {
            spacing: 10

            TextField {
                id: searchField
                anchors.leftMargin: 30
                anchors.verticalCenter: parent.verticalCenter
                width: searchBox.width * 0.45
                placeholderText: "Search..."
                color: Material.foreground
                focus: true

                onAccepted: searchUp(searchField.text, textArea.cursorPosition)
            }

            // TODO: custom component for buttons (and roundbuttons).
            Button {
                anchors.leftMargin: 30
                anchors.verticalCenter: parent.verticalCenter
                flat: true

                ToolTip.visible: hovered
                ToolTip.text: "Search up in the text"

                onClicked: searchUp(searchField.text, textArea.cursorPosition)

                Image {
                    width: 0.80 * parent.width
                    height: width
                    anchors.fill: parent
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/search-up"
                }
            }

            Button {
                anchors.verticalCenter: parent.verticalCenter
                flat: true

                ToolTip.visible: hovered
                ToolTip.text: "Search down in the text"

                onClicked: searchDown(searchField.text, textArea.cursorPosition)

                Image {
                    width: 0.80 * parent.width
                    height: width
                    anchors.fill: parent
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/search-down"
                }
            }

            Button {
                anchors.verticalCenter: parent.verticalCenter
                onClicked: root.commandHandler.execute(root.hostId, "logs", [root._unitId, searchField.text])

                ToolTip.visible: hovered
                ToolTip.text: "Show only matching rows"

                Image {
                    width: parent.width * 0.8
                    height: width
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/search"
                }
            }
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

        TextArea {
            id: textArea
            anchors.fill: parent
            readOnly: true
            activeFocusOnPress: false
            text: root.text
            font.family: "monospace"
            font.pointSize: 8

            cursorPosition: length - 1

            Rectangle {
                id: highlighter
                color: "#FF0000"
                radius: 5
                opacity: 0.4
                width: 0
                height: parent.font.pointSize * 2.0
                visible: false
            }
        }
    }

    Shortcut {
        sequence: StandardKey.Find
        onActivated: searchField.focus = true
    }

    Shortcut {
        sequence: StandardKey.FindNext
        onActivated: searchUp(searchField.text, textArea.cursorPosition)
    }

    Shortcut {
        sequence: StandardKey.FindPrevious
        onActivated: searchDown(searchField.text, textArea.cursorPosition)
    }


    function searchUp(query, currentPosition) {
        if (query.length === 0) {
            return;
        }

        let match = textArea.text.lastIndexOf(query, currentPosition - 1)
        if (match !== -1) {
            textArea.cursorPosition = match
            highlight(query)
        }
    }

    function searchDown(query, currentPosition) {
        if (query.length === 0) {
            return;
        }

        let match = textArea.text.indexOf(query, currentPosition + 1)
        if (match !== -1) {
            textArea.cursorPosition = match
            highlight(query)
        }
    }
    function highlight(query) {
        let cursor = textArea.cursorRectangle
        highlighter.x = cursor.x - 2
        // Adds some extra padding depending how much bigger the highlighter height is than the text.
        highlighter.y = cursor.y + ((highlighter.height - textArea.font.pixelSize) / 2.0 - 1)
        // With monospace font this crude approach will suffice.
        highlighter.width = (textArea.font.pixelSize - 4) * query.length
        highlighter.visible = true
    }

/*
    function highlight(text, search) {
        text.replace(search, `<font style="background-color: red">{search}</font>`)
    }

    function escapeHtml(text) {
        // TODO
        return text.replace(/&/g, "&amp;")
    }
    */
}