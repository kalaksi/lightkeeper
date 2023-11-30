import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse
import "../js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property string hostId
    property var _tabContents: {}


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()

    onHostIdChanged: {
        if (!(root.hostId in root._tabContents)) {
            openMainView(root.hostId)
        }
    }

    Component.onCompleted: {
        root._tabContents = {}
    }

    Connections {
        target: CommandHandler

        function onDetailsSubviewOpened(headerText, invocationId) {
            openTextView(headerText, invocationId)
        }

        function onLogsViewOpened(title, commandId, commandParams, invocationId) {
            let tabData = {
                "title": title,
                "component": logView.createObject(tabContent, {
                    hostId: root.hostId,
                    pendingInvocation: invocationId,
                    commandId: commandId,
                    commandParams: commandParams,
                })
            }

            createNewTab(tabData)
        }

        // For integrated text editor (not external).
        function onTextEditorSubviewOpened(commandId, invocationId, localFilePath) {
            openTextEditorView(commandId, invocationId, localFilePath)
        }

        // For integrated terminal.
        function onTerminalViewOpened(title, command) {
            let tabData = {
                "title": title,
                "component": terminalView.createObject(tabContent, {}),
            }

            createNewTab(tabData)
            tabData.component.open(command)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.backgroundColor
    }

    Header {
        id: mainViewHeader
        tabs: getTabTitles()
        showRefreshButton: true
        showMinimizeButton: true
        showMaximizeButton: true
        showCloseButton: true
        onRefreshClicked: CommandHandler.force_initialize_host(hostId)
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.closeClicked()

        onTabClosed: function(tabIndex) {
            root._tabContents[root.hostId][tabIndex].component.close()
            root.closeTab(tabIndex)
        }

        onTabIndexChanged: {
            if (root._tabContents[root.hostId][tabIndex] === undefined) {
                return
            }

            root._tabContents[root.hostId][tabIndex].component.focus()
        }
    }

    StackLayout {
        id: tabContent
        currentIndex: mainViewHeader.tabIndex
        anchors.top: mainViewHeader.bottom
        anchors.bottom: root.bottom
        anchors.left: root.left
        anchors.right: root.right
        anchors.margins: 10
    }

    Component {
        id: detailsMainView

        HostDetailsMainView {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }

    Component {
        id: textView

        HostDetailsTextView {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }

    Component {
        id: logView

        // TODO: disable button until service unit list is received.
        HostDetailsLogView {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }

    Component {
        id: textEditorView

        HostDetailsTextEditorView {
            Layout.fillWidth: true
            Layout.fillHeight: true

            // TODO: discard if not saving on close.
            onSaved: function(commandId, localFilePath, content) {
                let _invocationId = CommandHandler.saveAndUploadFile(
                    root.hostId,
                    commandId,
                    localFilePath,
                    content
                )
                root.closeSubview()
            }
        }
    }

    Component {
        id: terminalView

        HostDetailsTerminalView {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }

    Shortcut {
        sequence: StandardKey.Cancel
        onActivated: {
            root.closeClicked()
        }
    }

    Timer {
        id: hideSubContent
        interval: Theme.animation_duration()
        onTriggered: {
            textView.visible = false
            logView.visible = false
            textEditor.visible = false
            terminalView.close()
        }
    }

    function refresh() {
        root._tabContents[root.hostId][0].component.refresh()
    }

    function createNewTab(tabData) {
        if (!(root.hostId in root._tabContents)) {
            root._tabContents[root.hostId] = []
        }

        let similarTabs = root._tabContents[root.hostId].filter(tab => tab.title.startsWith(tabData.title)).length
        if (similarTabs > 0) {
            tabData.title = `${tabData.title} (${similarTabs + 1})`
        }

        // Extra padding for the title text before close button.
        tabData.title += "  "

        root._tabContents[root.hostId].push(tabData)

        let lastTabIndex = root._tabContents[root.hostId].length - 1
        mainViewHeader.tabs = getTabTitles()
        mainViewHeader.selectTab(lastTabIndex)
    }

    function closeTab(tabIndex) {
        let tabData = root._tabContents[root.hostId][tabIndex]
        tabData.component.destroy()
        root._tabContents[root.hostId].splice(tabIndex, 1)
        mainViewHeader.tabs = getTabTitles()
    }

    function openMainView(hostId) {
        let tabData = {
            "title": hostId,
            "component": detailsMainView.createObject(tabContent, {
                hostId: hostId,
            })
        }

        createNewTab(tabData)
    }

    function openTextView(title, invocationId) {
        let tabData = {
            "title": title,
            "component": textView.createObject(tabContent, {
                pendingInvocation: invocationId,
            })
        }

        createNewTab(tabData)
    }

    function openTextEditorView(commandId, invocationId, localFilePath) {
        let tabData = {
            "title": commandId,
            "component": textEditorView.createObject(tabContent, {
                commandId: commandId,
                localFilePath: localFilePath,
                pendingInvocation: invocationId,
            })
        }

        createNewTab(tabData)
    }

    function closeSubview() {
        root._showSubview = false
        hideSubContent.start()
    }

    function openSubview() {
        root._showSubview = true
    }

    function getTabTitles() {
        if (!(root.hostId in root._tabContents)) {
            return []
        }
        return root._tabContents[root.hostId].map(tabData => tabData.title)
    }
}