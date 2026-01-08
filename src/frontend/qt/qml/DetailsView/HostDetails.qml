pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

import Theme

import "../ChartsView"


Item {
    id: root
    required property string hostId
    property bool enableShortcuts: root.visible
    property bool showCharts: true
    property var _tabContents: {}
    property var _tabStacks: {}
    property bool _refreshingHost: false


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal customCommandsDialogOpened()


    onHostIdChanged: {
        if (!(root.hostId in root._tabContents)) {
            root._tabContents[root.hostId] = []
            root._tabStacks[root.hostId] = tabStack.createObject(tabStackContainer, {
                parentStackIndex: Object.keys(root._tabStacks).length,
            })

            // Create default tabs for host.
            if (root.showCharts) {
                createNewTab({
                    "title": "qrc:/main/images/button/charts",
                    "component": chartsView.createObject(root._tabStacks[root.hostId], {
                        hostId: hostId,
                    })
                }, false)
            }

            createNewTab({
                "title": hostId,
                "component": detailsMainView.createObject(root._tabStacks[root.hostId], {
                    hostId: hostId,
                })
            })
        }

        root.refresh()
    }

    Component.onCompleted: {
        root._tabContents = {}
        root._tabStacks = {}
    }

    Connections {
        target: LK.command

        function onTextViewOpened(title, invocationId) {
            let tabData = {
                "title": title,
                "component": textView.createObject(root._tabStacks[root.hostId], {
                    pendingInvocation: invocationId,
                })
            }

            root.createNewTab(tabData)
        }

        function onLogsViewOpened(showTimeControls, title, commandId, commandParams, invocationId) {
            let tabData = {
                "title": title,
                "component": logView.createObject(root._tabStacks[root.hostId], {
                    hostId: root.hostId,
                    pendingInvocation: invocationId,
                    commandId: commandId,
                    commandParams: commandParams,
                    showTimeControls: showTimeControls,
                })
            }

            root.createNewTab(tabData)
        }

        // For integrated text editor (not external).
        function onTextEditorViewOpened(commandId, invocationId, localFilePath) {
            let tabData = {
                "title": commandId,
                "component": textEditorView.createObject(root._tabStacks[root.hostId], {
                    commandId: commandId,
                    localFilePath: localFilePath,
                    pendingInvocation: invocationId,
                })
            }

            root.createNewTab(tabData)
        }

        // For integrated terminal.
        function onTerminalViewOpened(title, command) {
            let tabData = {
                "title": title,
                "component": terminalView.createObject(root._tabStacks[root.hostId], {})
            }

            root.createNewTab(tabData)
            tabData.component.open(command)
        }

        function onCommandOutputViewOpened(invocationId, title, text, errorText, progress) {
            let tabData = {
                "title": title,
                "component": commandOutputView.createObject(root._tabStacks[root.hostId], {
                    pendingInvocation: invocationId,
                    text: text,
                    errorText: errorText,
                    progress: progress,
                })
            }
            root.createNewTab(tabData)
        }
    }

    Connections {
        target: LK.hosts

        function onUpdateReceived(hostId) {
            if (hostId === root.hostId) {
                let inProgress = LK.hosts.getPendingMonitorCount(hostId) > 0
                if (inProgress && !root._refreshingHost) {
                    root._refreshingHost = true
                } else if (!inProgress && root._refreshingHost) {
                    root._refreshingHost = false

                    // Refresh charts tab after all monitoring data is received.
                    let chartsTabIndex = 0
                    if (root.showCharts && mainViewHeader.tabIndex === chartsTabIndex) {
                        root._tabContents[root.hostId][chartsTabIndex].component.refreshContent()
                    }
                }
            }
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.backgroundColor
    }

    Header {
        id: mainViewHeader
        tabs: root.getTabTitles()
        showMinimizeButton: true
        showMaximizeButton: true
        showRefreshButton: root.getCurrentTabContent() !== undefined && root.getCurrentTabContent().refreshContent !== undefined
        showSaveButton: root.getCurrentTabContent() !== undefined && root.getCurrentTabContent().save !== undefined
        showCharts: root.showCharts
        disableSaveButton: true

        onRefreshClicked: root.getCurrentTabContent().refreshContent()
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.close()
        onSaveClicked: root.getCurrentTabContent().save()

        onTabClosed: function(tabIndex) {
            root.closeTab(tabIndex)
        }

        onTabChanged: function(oldIndex, newIndex) {
            if (root._tabContents[root.hostId][newIndex] === undefined) {
                return
            }

            let oldComponent = root._tabContents[root.hostId][oldIndex]
            if (oldComponent !== undefined) {
                oldComponent.component.deactivate()
            }

            root._tabStacks[root.hostId].currentIndex = newIndex
            root._tabContents[root.hostId][newIndex].component.activate()
        }
    }

    StackLayout {
        id: tabStackContainer
        anchors.top: mainViewHeader.bottom
        anchors.bottom: root.bottom
        anchors.left: root.left
        anchors.right: root.right
        anchors.topMargin: Theme.spacingNormal
        anchors.leftMargin: Theme.spacingNormal
        anchors.rightMargin: Theme.spacingNormal
        anchors.bottomMargin: 1
    }

    Component {
        id: tabStack

        StackLayout {
            property int parentStackIndex: -1
        }
    }

    Component {
        id: detailsMainView

        HostDetailsMainView {
            onCustomCommandsDialogOpened: {
                root.customCommandsDialogOpened()
            }
        }
    }

    Component {
        id: textView

        HostDetailsTextView {
        }
    }

    Component {
        id: logView

        HostDetailsLogView {
        }
    }

    Component {
        id: textEditorView

        HostDetailsTextEditorView {
            onSaved: function(commandId, localFilePath, content) {
                pendingInvocation = LK.command.saveAndUploadFile(root.hostId, commandId, localFilePath, content)
            }
            onClosed: function(localFilePath) {
                LK.command.removeFile(localFilePath)
            }
            onContentChanged: function(localFilePath, content) {
                mainViewHeader.disableSaveButton = !LK.command.hasFileChanged(localFilePath, content)
            }
        }
    }

    Component {
        id: terminalView

        HostDetailsTerminalView {
        }
    }

    Component {
        id: chartsView

        ChartsView {
        }
    }

    Component {
        id: commandOutputView

        HostDetailsCommandOutputView {
        }
    }

    Shortcut {
        enabled: !(root.getCurrentTabContent() instanceof HostDetailsTerminalView) && root.enableShortcuts
        sequences: [StandardKey.Cancel]
        onActivated: root.close()
    }

    Shortcut {
        enabled: !(root.getCurrentTabContent() instanceof HostDetailsTerminalView) && root.enableShortcuts
        sequences: [StandardKey.Close]
        // Close current tab.
        onActivated: root.closeTab(mainViewHeader.tabIndex)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+1", "Ctrl+1"]
        onActivated: mainViewHeader.selectTab(0)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+2", "Ctrl+2"]
        onActivated: mainViewHeader.selectTab(1)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+3", "Ctrl+3"]
        onActivated: mainViewHeader.selectTab(2)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+4", "Ctrl+4"]
        onActivated: mainViewHeader.selectTab(3)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+5", "Ctrl+5"]
        onActivated: mainViewHeader.selectTab(4)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+6", "Ctrl+6"]
        onActivated: mainViewHeader.selectTab(5)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+7", "Ctrl+7"]
        onActivated: mainViewHeader.selectTab(6)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+8", "Ctrl+8"]
        onActivated: mainViewHeader.selectTab(7)
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequences: ["Alt+9", "Ctrl+9"]
        onActivated: mainViewHeader.selectTab(8)
    }

    Shortcut {
        enabled: !(root.getCurrentTabContent() instanceof HostDetailsTerminalView) && root.enableShortcuts
        sequences: [StandardKey.Refresh, "Ctrl+R"]
        onActivated: {
            let content = root.getCurrentTabContent()
            if (content !== undefined && content.refreshContent !== undefined) {
                content.refreshContent()
            }
        }
    }

    Shortcut {
        enabled: root.enableShortcuts
        sequence: "Ctrl+T"
        onActivated: LK.command.executeConfirmed("", root.hostId, "linux-shell", {})
    }


    function refresh() {
        // Refresh host details tab. (needed?)
        let hostTabIndex = root.showCharts ? 1 : 0
        root._tabContents[root.hostId][hostTabIndex].component.refresh()

        mainViewHeader.tabs = getTabTitles()
        tabStackContainer.currentIndex = root._tabStacks[root.hostId].parentStackIndex
        mainViewHeader.selectDefaultTab()
    }

    function createNewTab(tabData, selectTab = true) {
        let similarTabs = root._tabContents[root.hostId].filter(tab => tab.title.startsWith(tabData.title)).length
        if (similarTabs > 0) {
            tabData.title = `${tabData.title} (${similarTabs + 1})`
        }

        root._tabContents[root.hostId].push(tabData)
        let lastTabIndex = root._tabContents[root.hostId].length - 1
        mainViewHeader.tabs = getTabTitles()

        if (selectTab) {
            mainViewHeader.selectTab(lastTabIndex)
        }
    }

    function closeTab(tabIndex) {
        let tabData = root._tabContents[root.hostId][tabIndex]

        if (tabData.component.close === undefined) {
            return
        }

        tabData.component.close()
        tabData.component.destroy()
        root._tabContents[root.hostId].splice(tabIndex, 1)
        mainViewHeader.tabs = getTabTitles()

        mainViewHeader.selectDefaultTab()
    }

    function getTabTitles() {
        if (!(root.hostId in root._tabContents)) {
            return []
        }
        return root._tabContents[root.hostId].map(tabData => tabData.title)
    }

    function getCurrentTabContent() {
        if (!(root.hostId in root._tabContents)) {
            return undefined
        }
        let content = root._tabContents[root.hostId][mainViewHeader.tabIndex]
        if (content === undefined) {
            return undefined
        }
        else {
            return content.component
        }
    }

    function close() {
        for (let content of root._tabContents[root.hostId]) {
            content.component.deactivate()
        }

        root.closeClicked()
    }
}