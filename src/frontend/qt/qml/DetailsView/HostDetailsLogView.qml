/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../Text"
import "../Button"
import "../js/Utils.js" as Utils
import "../StyleOverride"


Item {
    id: root
    required property string hostId
    property bool showTimeControls: false
    property bool enableShortcuts: false
    property string commandId: ""
    property var commandParams: []
    property string text: ""
    property string errorText: ""
    property var pendingInvocation: 0
    property bool _loading: pendingInvocation > 0


    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            if (root.pendingInvocation === invocationId) {
                let commandResult = JSON.parse(commandResultJson)
                root.pendingInvocation = 0

                if (commandResult.error) {
                    root.errorText = commandResult.error
                }

                logList.rows = commandResult.message === "" ? [] : commandResult.message.split("\n")

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
                    placeholderTextColor: Theme.textColorDark
                    text: Utils.getLocalTimezoneISOString(Date.now() - 1 * 60 * 60 * 1000).replace("T", " ")
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
                    placeholderTextColor: Theme.textColorDark
                    text: "now"
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
                    placeholderTextColor: Theme.textColorDark
                    text: "1000"
                    onAccepted: root.refresh()
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
                        placeholderTextColor: Theme.textColorDark
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
                            onCheckedChanged: logList.toggleInvertRowOrder()

                            Layout.leftMargin: Theme.spacingTight
                            // Make more nicely centered with the label.
                            Layout.topMargin: 4
                        }


                        // Spacer
                        Rectangle {
                            Layout.fillWidth: true
                        }

                        SmallText {
                            id: searchDetails
                            text: ""
                            color: Theme.textColorDark

                            Layout.rightMargin: Theme.spacingLoose
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
                    width: 32
                    height: width

                    ToolTip.visible: hovered
                    ToolTip.text: "Search up"

                    onClicked: logList.search("up", searchField.text)

                    Image {
                        width: 0.80 * parent.width
                        height: width
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/arrow-up"
                    }
                }

                Button {
                    flat: true
                    width: 32
                    height: width

                    ToolTip.visible: hovered
                    ToolTip.text: "Search down"

                    onClicked: logList.search("down", searchField.text)

                    Image {
                        width: 0.80 * parent.width
                        height: width
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/arrow-down"
                    }
                }
            }
        }

        LogList {
            id: logList
            rows: []
            visible: rows.length > 0
            enableShortcuts: root.enableShortcuts
            searchText: searchField.text

            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        Item {
            visible: !root._loading && logList.rows.length === 0
            Layout.fillWidth: true
            Layout.fillHeight: true

            NormalText {
                text: "No logs available"
                color: Theme.textColorDark

                // Centered vertically and horizontally
                anchors.centerIn: parent
                anchors.verticalCenterOffset: -0.2 * parent.height
            }
        }

        // Loading animation
        Item {
            visible: root._loading
            Layout.fillWidth: true
            Layout.fillHeight: true

            WorkingSprite {
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: [StandardKey.Find, "/"]
        onActivated: {
            searchField.focus = true
            searchField.selectAll()
        }
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

    // Executes search again.
    function refresh() {
        logList.resetFields()

        if (root.showTimeControls) {
            // TODO: implement checkbox for "Use UTC timezone"
            // let fullStartTime = `${startTime.text} ${timezone.text}`
            // let fullEndTime = `${endTime.text} ${timezone.text}`
            let fullStartTime = startTime.text
            let fullEndTime = endTime.text

            root.pendingInvocation = LK.command.executePlain(
                root.hostId,
                root.commandId,
                [...root.commandParams, fullStartTime, fullEndTime, ""]
            )
        }
        else {
            root.pendingInvocation = LK.command.executePlain(
                root.hostId,
                root.commandId,
                [...root.commandParams, "", "", numberOfLines.text]
            )
        }
    }

    function activate() {
        searchField.focus = true
        root.enableShortcuts = true
    }

    function deactivate() {
        searchField.focus = false
        root.enableShortcuts = false
    }

    function close() {
        // Do nothing.
    }
}