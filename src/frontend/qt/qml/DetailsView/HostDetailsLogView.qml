import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

import "../Text"
import "../Button"


Item {
    // TODO: loading indicator and indicator (translucent text maybe) for empty results 
    // TODO: indicator for no search results
    id: root
    required property string hostId
    property string commandId: ""
    property string text: ""
    property string errorText: ""
    property string _unitId: ""
    property var pendingInvocations: []


    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocations.includes(commandResult.invocation_id)) {
                root.pendingInvocations = root.pendingInvocations.filter((invocationId) => invocationId != commandResult.invocationId)

                if (commandResult.error) {
                    root.errorText = commandResult.error
                }

                let oldListEnd = logList.rows.length - 1
                logList.setRows(commandResult.message.split("\n"))
                logList.currentIndex = oldListEnd

                let [rowsMatched, totalMatches] = logList.getSearchDetails()
                searchDetails.text = `${totalMatches} matches in ${rowsMatched} rows`
            }
        }
    }

    Rectangle {
        color: Material.background
        anchors.fill: parent
    }

    ColumnLayout {
        anchors.fill: parent

        Row {
            id: searchBox
            spacing: Theme.spacing_loose()

            Layout.topMargin: Theme.spacing_loose()
            Layout.leftMargin: root.width * 0.30
            Layout.fillWidth: true

            Row {
                spacing: Theme.spacing_loose()

                Column {
                    anchors.leftMargin: 30
                    spacing: Theme.spacing_normal()

                    TextField {
                        id: searchField
                        width: searchBox.width * 0.55
                        placeholderText: "Regex search..."
                        color: Material.foreground
                        focus: true

                        onAccepted: {
                            let [rowsMatched, totalMatches] = logList.search("up", searchField.text)
                            searchDetails.text = `${totalMatches} matches in ${rowsMatched} rows`
                        }
                    }

                    RowLayout {
                        width: searchField.width

                        SmallText {
                            id: searchDetails
                            text: ""
                            color: Theme.color_dark_text()
                        }

                        // Spacer
                        Rectangle {
                            Layout.fillWidth: true
                        }

                        SmallText {
                            text: `${logList.rows.length} total rows`
                            color: Theme.color_dark_text()
                        }
                    }
                }

                Button {
                    anchors.leftMargin: 30
                    flat: true

                    ToolTip.visible: hovered
                    ToolTip.text: "Search up"

                    onClicked: logList.search("up", searchField.text)

                    Image {
                        width: 0.80 * parent.width
                        height: width
                        anchors.fill: parent
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/search-up"
                    }
                }

                Button {
                    flat: true

                    ToolTip.visible: hovered
                    ToolTip.text: "Search down"

                    onClicked: logList.search("down", searchField.text)

                    Image {
                        width: 0.80 * parent.width
                        height: width
                        anchors.fill: parent
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/search-down"
                    }
                }

                Button {
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

        LogList {
            id: logList
            rows: root.text.split("\n")

            Layout.fillWidth: true
            Layout.fillHeight: true

            onLoadMore: function(pageNumber, pageSize) {
                CommandHandler.execute_confirmed(root.hostId, root.commandId, [pageNumber, pageSize])
            }
        }
    }

    Shortcut {
        sequence: [
            StandardKey.Find,
            // Vim-like shortcut:
            "Ctrl+7",
        ]
        onActivated: searchField.focus = true
    }

    Shortcut {
        sequence: [
            StandardKey.FindNext,
            // Vim-like shortcut:
            "Ctrl+N",
        ]
        onActivated: logList.search("down", searchField.text)
    }

    Shortcut {
        sequence: [
            StandardKey.FindPrevious,
            // Vim-like shortcut:
            "Ctrl+P",
        ]
        onActivated: logList.search("up", searchField.text)
    }

    function open(commandId, invocationId) {
        root.commandId = commandId
        root.pendingInvocations.push(invocationId)
        root.visible = true
        searchField.focus = true
    }

    function close() {
        if (root.visible) {
            root.visible = false
            reset()
        }
    }

    function reset() {
        root.text = ""
        root.errorText = ""
        root.pendingInvocations = []
        logList.reset()
    }
}