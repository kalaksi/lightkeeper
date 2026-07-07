/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

import Lightkeeper 1.0

import ".."
import "../ChartsView"


Item {
    id: root
    required property string hostId
    property bool enableShortcuts: root.visible
    property bool showCharts: true
    property var _tabContents: ({})
    property var _tabStacks: ({})
    property var _tabIndexByHost: ({})
    property string _previousHostId: ""
    property bool _refreshingHost: false
    // Bumped whenever a deferred tab's content is created, so bindings that read
    // getCurrentTabContent() (e.g. the refresh button) re-evaluate afterwards.
    property int _tabContentRevision: 0
    // Closed tab contents waiting to be destroyed.
    property var _pendingDestroy: []
    property var _pendingTabToCreate: null
    property string _pendingTabHostId: ""


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal customCommandsDialogOpened()


    onHostIdChanged: {
        if (root._previousHostId !== "" && root._previousHostId in root._tabContents) {
            let contents = root._tabContents[root._previousHostId]
            for (let content of contents) {
                if (content.component !== undefined && content.component.deactivate !== undefined) {
                    content.component.deactivate()
                }
            }
        }

        if (!(root.hostId in root._tabContents)) {
            root._tabContents[root.hostId] = []
            root._tabStacks[root.hostId] = tabStack.createObject(tabStackContainer, {
                parentStackIndex: Object.keys(root._tabStacks).length,
            })

            // Create default tabs for host.
            if (root.showCharts) {
                let chartsHostId = root.hostId
                root.createLazyTab("qrc:/main/images/button/charts", function(container) {
                    let component = chartsView.createObject(container, {
                        hostId: chartsHostId,
                    })
                    component.anchors.fill = container
                    return component
                }, false, false)
            }

            createNewTab({
                "title": hostId,
                "component": detailsMainView.createObject(root._tabStacks[root.hostId], {
                    hostId: hostId,
                })
            })
        }

        root.refresh()
        root._previousHostId = root.hostId
    }

    Component.onCompleted: {
        root._tabContents = {}
        root._tabStacks = {}
    }

    Connections {
        target: LK.command

        function onTextViewOpened(title, commandId, commandParams) {
            let tabHostId = root.hostId
            root.createLazyTab(title, function(container) {
                let component = textView.createObject(container, {
                    hostId: tabHostId,
                    commandId: commandId,
                    commandParams: commandParams,
                })
                component.anchors.fill = container
                return component
            })
        }

        function onLogsViewOpened(showTimeControls, title, commandId, commandParams) {
            let tabHostId = root.hostId
            root.createLazyTab(title, function(container) {
                let component = logView.createObject(container, {
                    hostId: tabHostId,
                    commandId: commandId,
                    commandParams: commandParams,
                    showTimeControls: showTimeControls,
                })
                component.anchors.fill = container
                return component
            })
        }

        // For integrated text editor (not external).
        function onTextEditorViewOpened(headerText, commandId, remoteFilePath) {
            let tabHostId = root.hostId
            root.createLazyTab(headerText, function(container) {
                let editorComponent = textEditorView.createObject(container, {
                    hostId: tabHostId,
                    commandId: commandId,
                    remoteFilePath: remoteFilePath,
                })
                editorComponent.anchors.fill = container
                editorComponent.closeTabRequested.connect(function() {
                    let contents = root._tabContents[tabHostId]
                    if (contents === undefined) {
                        return
                    }
                    for (const [index, tabContent] of contents.entries()) {
                        if (tabContent.component === editorComponent) {
                            root.closeTab(index)
                            return
                        }
                    }
                })
                return editorComponent
            })
        }

        // For integrated terminal.
        function onTerminalViewOpened(title, command) {
            let commandCopy = command.slice()
            root.createLazyTab(title, function(container) {
                let component = terminalView.createObject(container, {})
                component.anchors.fill = container
                component.open(commandCopy)
                return component
            })
        }

        // For file browser.
        function onFileBrowserOpened(directory) {
            let tabHostId = root.hostId
            root.createLazyTab("File browser", function(container) {
                let component = fileBrowserView.createObject(container, {
                    hostId: tabHostId,
                    initialPath: directory,
                })
                component.anchors.fill = container
                return component
            })
        }

        function onCommandOutputViewOpened(invocationId, title, text, errorText, progress) {
            root.createLazyTab(title, function(container) {
                let component = commandOutputView.createObject(container, {
                    pendingInvocation: invocationId,
                    text: text,
                    errorText: errorText,
                    progress: progress,
                })
                component.anchors.fill = container
                return component
            })
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
                    // Skip if its content hasn't been created (deferred) yet.
                    let chartsTabIndex = 0
                    let chartsTab = root._tabContents[root.hostId][chartsTabIndex]
                    if (root.showCharts && mainViewHeader.tabIndex === chartsTabIndex &&
                        chartsTab !== undefined && chartsTab.component !== undefined) {
                        chartsTab.component.refreshContent()
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
        showCharts: root.showCharts
        hostId: root.hostId

        onRefreshClicked: root.getCurrentTabContent().refreshContent()
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.close()

        onTabClosed: function(tabIndex) {
            root.closeTab(tabIndex)
        }

        onTabChanged: function(oldIndex, newIndex) {
            if (root._tabContents[root.hostId][newIndex] === undefined) {
                return
            }

            let oldComponent = root._tabContents[root.hostId][oldIndex]
            if (oldComponent !== undefined && oldComponent.component !== undefined && oldComponent.component.deactivate !== undefined) {
                oldComponent.component.deactivate()
            }

            root.activateCurrentTab()
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
        anchors.rightMargin: Theme.spacingTight
        anchors.bottomMargin: 1
    }

    Component {
        id: tabStack

        StackLayout {
            property int parentStackIndex: -1
        }
    }

    // Placeholder shown in the tab stack for tabs whose content is created lazily.
    // Displays a spinning progress animation until the real content is built, so
    // switching to a deferred tab feels instant.
    Component {
        id: lazyTabContainer

        Item {
            id: container
            property bool contentCreated: false

            WorkingSprite {
                visible: !container.contentCreated
                scale: 1.5
            }
        }
    }

    Timer {
        id: lazyCreateTimer
        // Give the tab strip/placeholder a frame to paint before constructing
        // the heavier tab body.
        interval: 16
        repeat: false
        onTriggered: root.createPendingTabContent()
    }

    // Closed tab contents are reparented here (out of the visible tab stack) and
    // destroyed shortly after by destroyTimer. Destroying heavy views (e.g. the
    // file browser's table views) can block the main thread for hundreds of ms,
    // so we keep it off the close interaction.
    Item {
        id: destroyGraveyard
        visible: false
    }

    Timer {
        id: destroyTimer
        // Long enough for the closed-tab frame to paint before the teardown
        // freeze; one item per tick so closing several tabs doesn't stack freezes.
        interval: 100
        repeat: true
        onTriggered: {
            if (root._pendingDestroy.length === 0) {
                stop()
                return
            }
            let entry = root._pendingDestroy.shift()
            if (entry !== null && entry !== undefined) {
                if (entry.phase === "close") {
                    if (entry.component !== null && entry.component !== undefined) {
                        entry.component.close()
                    }
                    entry.phase = "destroy"
                    root._pendingDestroy.push(entry)
                    return
                }
                if (entry.stackItem !== null && entry.stackItem !== undefined) {
                    entry.stackItem.destroy()
                }
            }
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

        HostDetailsCodeEditorView {
            onSaved: function(commandId, remoteFilePath, content) {
                pendingInvocation = LK.command.saveAndUploadFile(root.hostId, commandId, remoteFilePath, content)
            }
            onClosed: function(remoteFilePath) {
                LK.command.removeCachedFile(root.hostId, remoteFilePath)
            }
        }
    }

    Component {
        id: terminalView

        HostDetailsTerminalView {
        }
    }

    Component {
        id: fileBrowserView

        HostDetailsFileBrowserView {
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
        enabled: !(root.getCurrentTabContent() instanceof HostDetailsTerminalView) && 
                 !(root.getCurrentTabContent() instanceof HostDetailsCodeEditorView) && 
                 root.enableShortcuts
        sequences: [StandardKey.Cancel]
        onActivated: root.close()
    }

    Shortcut {
        enabled: !(root.getCurrentTabContent() instanceof HostDetailsTerminalView) && 
                 !(root.getCurrentTabContent() instanceof HostDetailsCodeEditorView) && 
                 root.enableShortcuts
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
        enabled: !(root.getCurrentTabContent() instanceof HostDetailsTerminalView) && 
                 !(root.getCurrentTabContent() instanceof HostDetailsCodeEditorView) && 
                 root.enableShortcuts
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

    Shortcut {
        enabled: root.enableShortcuts && 
                 !(root.getCurrentTabContent() instanceof HostDetailsCodeEditorView)
        sequence: "Ctrl+Y"
        onActivated: LK.command.executeConfirmed("", root.hostId, "_internal-filebrowser-ls", ["/"])
    }


    function refresh() {
        // Refresh host details tab. (needed?)
        let hostTabIndex = root.showCharts ? 1 : 0
        root._tabContents[root.hostId][hostTabIndex].component.refresh()

        let savedIdx = getLastTabIndex()
        mainViewHeader.tabs = getTabTitles()
        tabStackContainer.currentIndex = root._tabStacks[root.hostId].parentStackIndex
        mainViewHeader.selectTab(savedIdx)
        root.activateCurrentTab()
    }

    function activateCurrentTab() {
        if (!(root.hostId in root._tabContents)) {
            return
        }
        let tabIndex = mainViewHeader.tabIndex
        let tabData = root._tabContents[root.hostId][tabIndex]
        if (tabData === undefined) {
            return
        }
        // Show the tab (placeholder for deferred tabs) immediately.
        root._tabStacks[root.hostId].currentIndex = tabIndex
        root._tabIndexByHost[root.hostId] = tabIndex

        // Lazily create deferred content shortly after the tab is activated.
        if (tabData.component === undefined && tabData.componentProvider !== undefined) {
            root.scheduleTabContentCreation(root.hostId, tabData)
            return
        }

        if (tabData.component !== undefined && tabData.component.activate !== undefined) {
            tabData.component.activate()
        }
    }

    function getLastTabIndex() {
        let defaultTabIndex = root.showCharts ? 1 : 0
        let tabCount = root._tabContents[root.hostId].length
        let lastTabIndex = root._tabIndexByHost[root.hostId]
        let result = (lastTabIndex !== undefined && lastTabIndex >= 0 && lastTabIndex < tabCount) ? lastTabIndex : defaultTabIndex
        return result
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

        return tabData
    }

    function createLazyTab(title, componentProvider, selectTab = true, canClose = true) {
        let container = lazyTabContainer.createObject(root._tabStacks[root.hostId])
        return root.createNewTab({
            "title": title,
            "container": container,
            "component": undefined,
            "componentProvider": componentProvider,
            "canClose": canClose,
        }, selectTab)
    }

    function scheduleTabContentCreation(hostId, tabData) {
        root._pendingTabHostId = hostId
        root._pendingTabToCreate = tabData
        lazyCreateTimer.restart()
    }

    function createPendingTabContent() {
        let targetHostId = root._pendingTabHostId
        let tabData = root._pendingTabToCreate
        root._pendingTabHostId = ""
        root._pendingTabToCreate = null

        // Bail if the pending creation is stale (host switched, tab closed/replaced, etc.)
        if (tabData == null ||
            root.hostId !== targetHostId ||
            !(targetHostId in root._tabContents) ||
            root._tabContents[targetHostId][mainViewHeader.tabIndex] !== tabData) {

            return
        }

        // Nothing to do if it's already built or has no provider.
        if (tabData.component !== undefined || tabData.componentProvider === undefined) {
            return
        }

        let component = tabData.componentProvider(tabData.container)
        if (component == null) {
            return
        }

        tabData.component = component
        if (tabData.container !== undefined) {
            tabData.container.contentCreated = true
        }
        if (tabData.component.activate !== undefined) {
            tabData.component.activate()
        }
        // Notify bindings that depend on the active tab's content.
        root._tabContentRevision++
    }

    function closeTab(tabIndex) {
        let tabData = root._tabContents[root.hostId][tabIndex]

        if (tabData === undefined || tabData.canClose === false) {
            return
        }
        if (tabData.component !== undefined && tabData.component.close === undefined) {
            return
        }

        // The item actually parented in the visible tab stack: the lazy container
        // if present, otherwise the component itself.
        let stackItem = tabData.container !== undefined ? tabData.container : tabData.component

        // Detach from the tab stack right now so the close is instant, then let
        // destroyTimer free it a beat later. Destroying it inline (or even via
        // Qt.callLater, which runs before the next paint) makes the close itself
        // freeze for the full teardown cost.
        if (stackItem !== undefined && stackItem !== null) {
            stackItem.visible = false
            stackItem.parent = destroyGraveyard
            root._pendingDestroy.push({
                "phase": "close",
                "component": tabData.component,
                "stackItem": stackItem,
            })
        }

        root._tabContents[root.hostId].splice(tabIndex, 1)
        mainViewHeader.tabs = getTabTitles()

        let tabCount = root._tabContents[root.hostId].length
        if (tabCount > 0) {
            mainViewHeader.selectTab(Math.min(tabIndex, tabCount - 1))
        }

        destroyTimer.start()
    }

    function getTabTitles() {
        if (!(root.hostId in root._tabContents)) {
            return []
        }
        return root._tabContents[root.hostId].map(tabData => tabData.title)
    }

    function getCurrentTabContent() {
        // Establish a binding dependency so callers re-evaluate when deferred
        // content is created (see _tabContentRevision).
        void root._tabContentRevision
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
            if (content.component !== undefined && content.component.deactivate !== undefined) {
                content.component.deactivate()
            }
        }

        root.closeClicked()
    }
}