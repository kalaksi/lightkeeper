import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15

import ".."
import "../Text"
import "../Button"
import "../js/Utils.js" as Utils


Item {
    // TODO: loading indicator and indicator (translucent text maybe) for empty results 
    // TODO: indicator for no search results
    id: root
    required property string hostId
    property bool showTimeControls: false
    property string commandId: ""
    property var commandParams: []
    property string text: ""
    property string errorText: ""
    property string _unitId: ""
    property var pendingInvocation: 0


    Connections {
        target: HostDataManager

        function onCommand_result_received(commandResultJson) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === commandResult.invocation_id) {
                root.pendingInvocation = 0

                if (commandResult.error) {
                    root.errorText = commandResult.error
                }

                let rows = commandResult.message.split("\n")
                rows.reverse()
                logList.rows = rows

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
                    visible: root.showTimeControls
                    height: parent.height
                    text: "From"
                }

                TextField {
                    id: startTime
                    visible: root.showTimeControls
                    width: searchBox.width * 0.12
                    placeholderText: "Start date"
                    text: Utils.getLocalTimezoneISOString(Date.now() - 1 * 60 * 60 * 1000).replace("T", " ")
                    color: Theme.textColor
                    onAccepted: timeRangeSubmit.clicked(null)
                }

                NormalText {
                    visible: root.showTimeControls
                    text: "To"
                }

                TextField {
                    id: endTime
                    visible: root.showTimeControls
                    width: searchBox.width * 0.12
                    placeholderText: "End date"
                    text: "now"
                    color: Theme.textColor
                    onAccepted: timeRangeSubmit.clicked(null)
                }

                TextField {
                    visible: false
                    id: timezone
                    text: Utils.formatTimezone(new Date().getTimezoneOffset())
                }

                NormalText {
                    visible: !root.showTimeControls
                    text: "Lines fo fetch"
                }

                TextField {
                    id: numberOfLines
                    visible: !root.showTimeControls
                    width: searchBox.width * 0.12
                    placeholderText: "Number of lines"
                    text: "1000"
                    onAccepted: numberOfLinesSubmit.clicked(null)
                }

                ImageButton {
                    id: numberOfLinesSubmit
                    visible: !root.showTimeControls
                    size: numberOfLines.height
                    imageSource: "qrc:/main/images/button/search"
                    tooltip: "Fetch"
                    onClicked: root.refresh()
                }

                ImageButton {
                    id: timeRangeSubmit
                    visible: root.showTimeControls
                    size: numberOfLines.height
                    imageSource: "qrc:/main/images/button/search"
                    tooltip: "Apply time range"
                    onClicked: root.refresh()
                }

                // Spacer
                Item {
                    width: root.showTimeControls ? 0.01 * searchBox.width : 0.08 * searchBox.width
                    height: searchBox.height
                }

                Column {
                    spacing: Theme.spacingNormal

                    TextField {
                        id: searchField
                        width: searchBox.width * 0.55
                        placeholderText: "Regex search..."
                        color: Theme.textColor
                        focus: true

                        onAccepted: {
                            focus = false
                            logList.focus = true
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
            }
        }

        LogList {
            id: logList
            rows: root.text.split("\n")
            visible: rows.length > 0

            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        // Loading animation
        Item {
            visible: logList.rows.length === 0

            WorkingSprite {
                visible: root.pendingInvocation > 0
                id: workingSprite
            }

            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }

    Shortcut {
        sequence: StandardKey.Find
        onActivated: searchField.focus = true
    }

    Shortcut {
        sequence: StandardKey.FindNext
        onActivated: logList.search("down", searchField.text)
    }

    Shortcut {
        sequence: StandardKey.FindPrevious
        onActivated: logList.search("up", searchField.text)
    }

    // Executes searchagain.
    function refresh() {
        if (root.showTimeControls) {
            // TODO: implement checkbox for "Use UTC timezone"
            // let fullStartTime = `${startTime.text} ${timezone.text}`
            // let fullEndTime = `${endTime.text} ${timezone.text}`
            let fullStartTime = startTime.text
            let fullEndTime = endTime.text

            root.pendingInvocation = CommandHandler.executePlain(
                root.hostId,
                root.commandId,
                [...root.commandParams, fullStartTime, fullEndTime, "", ""]
            )
        }
        else {
            root.pendingInvocation = CommandHandler.executePlain(
                root.hostId,
                root.commandId,
                [...root.commandParams, "", "", "", ""]
            )
        }
    }

    function focus() {
        searchField.focus = true
    }

    function close() {
        // Do nothing.
    }
}