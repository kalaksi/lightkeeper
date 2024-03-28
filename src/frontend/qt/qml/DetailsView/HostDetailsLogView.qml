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
    property bool enableShortcuts: false
    property string commandId: ""
    property var commandParams: []
    property string text: ""
    property string errorText: ""
    property string _unitId: ""
    property var pendingInvocation: 0


    Connections {
        target: HostDataManager

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === invocationId) {
                root.pendingInvocation = 0

                if (commandResult.error) {
                    root.errorText = commandResult.error
                }

                let rows = commandResult.message.split("\n")
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

            Layout.topMargin: Theme.spacingNormal
            Layout.bottomMargin: Theme.spacingNormal
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
                    spacing: Theme.spacingTight

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
                        spacing: Theme.spacingNormal
                        width: searchField.width

                        NormalText {
                            text: "Invert row order"
                        }

                        Switch {
                            id: invertRowOrder
                            checked: true
                            onCheckedChanged: logList.invertRowOrder()

                            Layout.leftMargin: Theme.spacingTight
                            // Make more nicely centered with the label.
                            Layout.topMargin: 4
                        }


                        // Spacer
                        Rectangle {
                            Layout.fillWidth: true
                        }

                        SmallText {
                            Layout.rightMargin: Theme.spacingLoose

                            id: searchDetails
                            text: ""
                            color: Theme.textColorDark
                        }

                        SmallText {
                            text: `row ${logList.currentIndex + 1} of ${logList.rows.length}`
                            color: Theme.textColorDark
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
            visible: rows.length > 0
            rows: root.text.split("\n")
            enableShortcuts: root.enableShortcuts

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
        enabled: root.enableShortcuts
        sequences: [StandardKey.Find, "/"]
        onActivated: searchField.focus = true
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: StandardKey.FindNext
        onActivated: logList.search("down", searchField.text)
    }

    Shortcut {
        enabled: root.enableShortcuts
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
        root.enableShortcuts = true
    }

    function unfocus() {
        searchField.focus = false
        root.enableShortcuts = false
    }

    function close() {
        // Do nothing.
    }
}