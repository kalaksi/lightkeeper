/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts

import Lightkeeper 1.0
import Lighthouse.LazyTabStack 1.0

import ".."
import "../ChartsView"


Item {
    id: root
    required property string hostId
    property bool enableShortcuts: root.visible
    property bool showCharts: true
    property var _tabStacks: ({})
    property var _tabIndexByHost: ({})
    property string _previousHostId: ""
    property bool _refreshingHost: false


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal customCommandsDialogOpened()
    signal categoryConfigDialogOpened(string categoryName)


    onHostIdChanged: {
        if (root._previousHostId !== "" && root._previousHostId in root._tabStacks) {
            root._tabStacks[root._previousHostId].deactivateAll()
        }

        if (!(root.hostId in root._tabStacks)) {
            let stack = hostTabStackComponent.createObject(tabStackContainer, {
                parentStackIndex: tabStackContainer.children.length,
            })
            root._tabStacks[root.hostId] = stack

            // Create default tabs for host.
            if (root.showCharts) {
                let chartsHostId = root.hostId
                stack.addLazyTab("qrc:/main/images/button/charts", function(shell) {
                    let component = chartsView.createObject(shell, {
                        hostId: chartsHostId,
                    })
                    component.anchors.fill = shell
                    return component
                }, {
                    select: false,
                    canClose: false,
                })
            }

            stack.addTab(hostId, detailsMainView.createObject(null, {
                hostId: hostId,
            }), {
                select: false,
            })
        }

        root.refresh()
        root._previousHostId = root.hostId
    }

    Connections {
        target: LK.command

        function onTextViewOpened(title, commandId, commandParams) {
            let tabHostId = root.hostId
            let stack = root._tabStacks[tabHostId]
            stack.addLazyTab(root.uniqueTabTitle(stack, title), function(shell) {
                let component = textView.createObject(shell, {
                    hostId: tabHostId,
                    commandId: commandId,
                    commandParams: commandParams,
                })
                component.anchors.fill = shell
                return component
            })
        }

        function onLogsViewOpened(showTimeControls, title, commandId, commandParams) {
            let tabHostId = root.hostId
            let stack = root._tabStacks[tabHostId]
            stack.addLazyTab(root.uniqueTabTitle(stack, title), function(shell) {
                let component = logView.createObject(shell, {
                    hostId: tabHostId,
                    commandId: commandId,
                    commandParams: commandParams,
                    showTimeControls: showTimeControls,
                })
                component.anchors.fill = shell
                return component
            })
        }

        // For integrated text editor (not external).
        function onTextEditorViewOpened(headerText, commandId, remoteFilePath) {
            let tabHostId = root.hostId
            let stack = root._tabStacks[tabHostId]
            stack.addLazyTab(root.uniqueTabTitle(stack, headerText), function(shell) {
                let editorComponent = textEditorView.createObject(shell, {
                    hostId: tabHostId,
                    commandId: commandId,
                    remoteFilePath: remoteFilePath,
                })
                editorComponent.anchors.fill = shell
                editorComponent.closeTabRequested.connect(function() {
                    if (tabHostId in root._tabStacks) {
                        root._tabStacks[tabHostId].closeTabByContent(editorComponent)
                    }
                })
                return editorComponent
            })
        }

        // For integrated terminal.
        function onTerminalViewOpened(title, command) {
            let commandCopy = command.slice()
            let stack = root._tabStacks[root.hostId]
            stack.addLazyTab(root.uniqueTabTitle(stack, title), function(shell) {
                let component = terminalView.createObject(shell, {})
                component.anchors.fill = shell
                component.open(commandCopy)
                return component
            })
        }

        // For file browser.
        function onFileBrowserOpened(directory) {
            let tabHostId = root.hostId
            let stack = root._tabStacks[tabHostId]
            stack.addLazyTab(root.uniqueTabTitle(stack, "File browser"), function(shell) {
                let component = fileBrowserView.createObject(shell, {
                    hostId: tabHostId,
                    initialPath: directory,
                })
                component.anchors.fill = shell
                return component
            })
        }

        function onCommandOutputViewOpened(invocationId, title, text, errorText, progress) {
            let stack = root._tabStacks[root.hostId]
            stack.addLazyTab(root.uniqueTabTitle(stack, title), function(shell) {
                let component = commandOutputView.createObject(shell, {
                    pendingInvocation: invocationId,
                    text: text,
                    errorText: errorText,
                    progress: progress,
                })
                component.anchors.fill = shell
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
                    let stack = root._tabStacks[root.hostId]
                    let chartsContent = stack.contentAt(chartsTabIndex)
                    if (root.showCharts && mainViewHeader.tabIndex === chartsTabIndex &&
                        chartsContent !== null) {
                        chartsContent.refreshContent()
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
        showMinimizeButton: true
        showMaximizeButton: true
        showRefreshButton: {
            const content = root.getCurrentTabContent()
            return content !== undefined && content.refreshContent !== undefined
        }
        showCharts: root.showCharts
        hostId: root.hostId

        onRefreshClicked: {
            const content = root.getCurrentTabContent()
            if (content !== undefined && content.refreshContent !== undefined) {
                content.refreshContent()
            }
        }
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.close()

        onTabClosed: function(tabIndex) {
            root.closeTab(tabIndex)
        }

        onTabChanged: function(oldIndex, newIndex) {
            if (!(root.hostId in root._tabStacks)) {
                return
            }
            let stack = root._tabStacks[root.hostId]
            if (newIndex < 0 || newIndex >= stack.tabTitles.length) {
                return
            }
            stack.selectTab(newIndex)
            root._tabIndexByHost[root.hostId] = newIndex
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
        id: workingSpriteIndicator

        WorkingSprite {
            scale: 1.5
        }
    }

    Component {
        id: hostTabStackComponent

        LazyTabStack {
            property int parentStackIndex: -1
            loadingIndicator: workingSpriteIndicator

            onCurrentIndexChanged: {
                if (root.hostId in root._tabStacks && root._tabStacks[root.hostId] === this) {
                    mainViewHeader.selectTab(currentIndex)
                }
            }

            onTabTitlesChanged: {
                if (root.hostId in root._tabStacks && root._tabStacks[root.hostId] === this) {
                    mainViewHeader.setTabs(tabTitles, currentIndex)
                }
            }

            onContentActivated: function(content) {
                if (content.activate !== undefined) {
                    content.activate()
                }
            }
            onContentDeactivated: function(content) {
                if (content.deactivate !== undefined) {
                    content.deactivate()
                }
            }
            onContentClosing: function(content) {
                if (content.close !== undefined) {
                    content.close()
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
            onCategoryConfigDialogOpened: function(categoryName) {
                root.categoryConfigDialogOpened(categoryName)
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
        if (!(root.hostId in root._tabStacks)) {
            return
        }
        let stack = root._tabStacks[root.hostId]
        let hostTabIndex = root.showCharts ? 1 : 0
        let hostContent = stack.contentAt(hostTabIndex)
        if (hostContent !== null) {
            hostContent.refresh()
        }

        let savedIdx = getLastTabIndex()
        tabStackContainer.currentIndex = stack.parentStackIndex
        mainViewHeader.setTabs(stack.tabTitles, savedIdx)
        stack.selectTab(savedIdx)
        root._tabIndexByHost[root.hostId] = savedIdx
    }

    function getLastTabIndex() {
        let defaultTabIndex = root.showCharts ? 1 : 0
        let stack = root._tabStacks[root.hostId]
        let tabCount = stack.tabTitles.length
        let lastTabIndex = root._tabIndexByHost[root.hostId]
        let result = (lastTabIndex !== undefined && lastTabIndex >= 0 && lastTabIndex < tabCount) ? lastTabIndex : defaultTabIndex
        return result
    }

    function closeTab(tabIndex) {
        if (root.hostId in root._tabStacks) {
            root._tabStacks[root.hostId].closeTab(tabIndex)
        }
    }

    function uniqueTabTitle(stack, title) {
        const similarTabs = stack.tabTitles.filter(tabTitle => tabTitle.startsWith(title)).length
        if (similarTabs > 0) {
            return `${title} (${similarTabs + 1})`
        }
        return title
    }

    function getCurrentTabContent() {
        if (!(root.hostId in root._tabStacks)) {
            return undefined
        }
        const content = root._tabStacks[root.hostId].currentContent
        return content === null ? undefined : content
    }

    function close() {
        if (root.hostId in root._tabStacks) {
            root._tabStacks[root.hostId].deactivateAll()
        }

        root.closeClicked()
    }
}
