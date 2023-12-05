import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import ".."
import "../Text"
import "../Button"


Item {
    // TODO: loading indicator and indicator (translucent text maybe) for empty results 
    // TODO: indicator for no search results
    id: root
    required property string hostId
    property string commandId: ""
    property var commandParams: []
    property string text: ""
    property string errorText: ""
    property string _unitId: ""
    property var pendingInvocation: -1


    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === commandResult.invocation_id) {
                root.pendingInvocation = -1

                if (commandResult.error) {
                    root.errorText = commandResult.error
                }

                // TODO: handle better situations where more log lines have appeared and so this isn't an accurate position
                let oldListEnd = logList.rows.length - 1
                let rows = commandResult.message.split("\n")
                rows.reverse()
                logList.rows = rows
                logList.currentIndex = oldListEnd

                let [rowsMatched, totalMatches] = logList.getSearchDetails()
                searchDetails.text = `${totalMatches} matches in ${rowsMatched} rows`
            }
        }
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingNormal

        Row {
            id: searchBox
            spacing: Theme.spacingLoose

            Layout.topMargin: Theme.spacingLoose
            Layout.fillWidth: true

            Row {
                spacing: Theme.spacingLoose

                NormalText {
                    height: parent.height
                    text: "From"
                }

                TextField {
                    id: startDate
                    width: searchBox.width * 0.12
                    placeholderText: "Start date"
                    // default is yesterday at 00:00:00
                    text: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString().split("T")[0] + " 00:00:00"
                    color: Theme.textColor
                }

                NormalText {
                    text: "To"
                }

                TextField {
                    id: endDate
                    width: searchBox.width * 0.12
                    placeholderText: "End date"
                    // default is today at 23:59:59
                    text: new Date(Date.now()).toISOString().split("T")[0] + " 23:59:59"
                    color: Theme.textColor
                }


                Column {
                    anchors.leftMargin: 30
                    spacing: Theme.spacingNormal

                    TextField {
                        id: searchField
                        width: searchBox.width * 0.55
                        placeholderText: "Regex search..."
                        color: Theme.textColor
                        focus: true

                        onAccepted: {
                            let [rowsMatched, totalMatches] = logList.search("down", searchField.text)
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
            visible: rows.length > 0

            Layout.fillWidth: true
            Layout.fillHeight: true

            onLoadMore: function(pageNumber, pageSize) {
                CommandHandler.execute_confirmed(
                    root.hostId,
                    root.commandId,
                    [...root.commandParams, startDate.text, endDate.text, pageNumber, pageSize]
                )
            }
        }

        // Loading animation
        Item {
            visible: logList.rows.length === 0

            WorkingSprite {
                visible: root.pendingInvocation > -1
                id: workingSprite
            }

            Layout.fillWidth: true
            Layout.fillHeight: true
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


    function focus() {
        searchField.focus = true
    }

    function close() {
        // Do nothing.
    }
}