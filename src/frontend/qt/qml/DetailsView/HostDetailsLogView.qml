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
    property var _matches: []
    property string _lastQuery: ""


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
                // TODO: use invocation id (only needed for dialog?)?
                root.commandHandler.execute(root.hostId, "logs", [root._unitId])
            }
        }

        Row {
            spacing: 10

            TextField {
                id: searchField
                anchors.leftMargin: 30
                anchors.verticalCenter: parent.verticalCenter
                width: searchBox.width * 0.55
                placeholderText: "RegExp search..."
                color: Material.foreground
                focus: true

                onAccepted: search("up", searchField.text)
            }

            // TODO: custom component for buttons (and roundbuttons).
            Button {
                anchors.leftMargin: 30
                anchors.verticalCenter: parent.verticalCenter
                flat: true

                ToolTip.visible: hovered
                ToolTip.text: "Search up in the text"

                onClicked: search("up", searchField.text)

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

                onClicked: search("down", searchField.text)

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
                onClicked: searchRows(searchField.text)

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
        onActivated: search("up", searchField.text)
    }

    Shortcut {
        sequence: StandardKey.FindPrevious
        onActivated: search("down", searchField.text)
    }


    function search(direction, query) {
        dehighlight()
        if (query.length === 0) {
            return;
        }

        if (root._matches.length === 0 || root._lastQuery !== query) {
            recordMatches(query, textArea.text)
            root._lastQuery = query
        }

        let match;
        if (direction === "up") {
            let reversed = [...root._matches].reverse()
            match = reversed.find((item) => textArea.cursorPosition > item[0])
        }
        else if (direction === "down") {
            match = root._matches.find((item) => textArea.cursorPosition < item[0])
        }

        if (match) {
            textArea.cursorPosition = match[0]
            highlight(match[1])
        }

    }

    function recordMatches(query, text) {
        root._matches = []
        let regexp = RegExp(query, "g")

        let match = regexp.exec(text)
        while (match !== null) {
            root._matches.push([match.index, match[0]])
            match = regexp.exec(text)
        }
    }

    function searchRows(query) {
        dehighlight()
        if (query.length === 0) {
            return;
        }

        root.commandHandler.execute(root.hostId, "logs", [root._unitId, query])
    }

    function highlight(match) {
        let cursor = textArea.cursorRectangle
        highlighter.x = cursor.x - 2
        // Adds some extra padding depending how much bigger the highlighter height is than the text.
        highlighter.y = cursor.y + ((highlighter.height - textArea.font.pixelSize) / 2.0 - 1)
        // With monospace font this crude approach will suffice for now (TODO: better solution)
        highlighter.width = (textArea.font.pixelSize - 4.8) * match.length + 4
        highlighter.visible = true
    }

    function dehighlight() {
        highlighter.visible = false
    }
}