/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import ".."
import "../StyleOverride"

LightkeeperDialog {
    id: root
    property string groupName: ""
    property bool readOnly: false
    property bool allowModuleAddRemove: !root.readOnly
    /// Module ID as key, array of settings as value.
    property var groupConnectorSettings: ({})
    property var groupMonitorSettings: ({})
    property var groupCommandSettings: ({})
    property var _connectorList: []
    property var _monitorList: []
    property var _commandList: []
    property bool _loading: true
    property int _buttonSize: 26

    title: `Configuration group: ${root.groupName}`
    implicitWidth: 630
    implicitHeight: 700
    standardButtons: Dialog.Cancel | Dialog.Ok
    signal configurationChanged()

    Component.onCompleted: {
        resetModel()
    }

    onOpened: {
        root.refreshModel()
        root._loading = false
    }

    onAccepted: {
        if (!root.readOnly) {
            let connectorSettingsJson = JSON.stringify(root.groupConnectorSettings)
            let monitorSettingsJson = JSON.stringify(root.groupMonitorSettings)
            let commandSettingsJson = JSON.stringify(root.groupCommandSettings)

            LK.config.updateGroupModules(root.groupName, connectorSettingsJson, monitorSettingsJson, commandSettingsJson)
            LK.config.writeGroupConfiguration()
        }

        resetModel()
    }

    onRejected: {
        resetModel()
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: scrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root._loading
    }

    contentItem: ScrollView {
        id: scrollView
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        contentWidth: availableWidth
        clip: true

        ColumnLayout {
            id: rootColumn
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.marginScrollbar
            spacing: Theme.spacingTight

            ModuleConfigSection {
                title: "Connector module settings"
                moduleIds: root._connectorList
                settingsByModule: root.groupConnectorSettings
                readOnly: root.readOnly
                allowAddRemove: root.allowModuleAddRemove
                groupName: root.groupName
                moduleType: "connector"
                addDialogLabel: "Connector module"
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onRemoveModule: function(moduleId) {
                    delete root.groupConnectorSettings[moduleId]
                    root._connectorList = root._connectorList.filter((id) => id !== moduleId)
                }
                onAddModule: function(moduleId) {
                    root.groupConnectorSettings[moduleId] =
                        LK.config.getGroupModuleSettings(root.groupName, moduleId).map(JSON.parse)
                    root._connectorList = root._connectorList.concat(moduleId).sort()
                }
                onUpdateModuleSettings: function(moduleId, settings) {
                    root.groupConnectorSettings[moduleId] = settings
                    let temp = root._connectorList
                    root._connectorList = []
                    root._connectorList = temp
                }
            }

            ModuleConfigSection {
                title: "Enabled monitoring modules and settings"
                moduleIds: root._monitorList
                settingsByModule: root.groupMonitorSettings
                readOnly: root.readOnly
                allowAddRemove: root.allowModuleAddRemove
                showSudoBadge: true
                groupName: root.groupName
                moduleType: "monitor"
                addDialogLabel: "Monitoring module"
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onRemoveModule: function(moduleId) {
                    delete root.groupMonitorSettings[moduleId]
                    root._monitorList = root._monitorList.filter((id) => id !== moduleId)
                }
                onAddModule: function(moduleId) {
                    root.groupMonitorSettings[moduleId] =
                        LK.config.getGroupModuleSettings(root.groupName, moduleId).map(JSON.parse)
                    root._monitorList = root._monitorList.concat(moduleId).sort()
                }
                onUpdateModuleSettings: function(moduleId, settings) {
                    root.groupMonitorSettings[moduleId] = settings
                    let temp = root._monitorList
                    root._monitorList = []
                    root._monitorList = temp
                }
            }

            ModuleConfigSection {
                title: "Enabled command modules and settings"
                moduleIds: root._commandList
                settingsByModule: root.groupCommandSettings
                readOnly: root.readOnly
                allowAddRemove: root.allowModuleAddRemove
                showSudoBadge: true
                groupName: root.groupName
                moduleType: "command"
                addDialogLabel: "Command module"
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onRemoveModule: function(moduleId) {
                    delete root.groupCommandSettings[moduleId]
                    root._commandList = root._commandList.filter((id) => id !== moduleId)
                }
                onAddModule: function(moduleId) {
                    root.groupCommandSettings[moduleId] =
                        LK.config.getGroupModuleSettings(root.groupName, moduleId).map(JSON.parse)
                    root._commandList = root._commandList.concat(moduleId).sort()
                }
                onUpdateModuleSettings: function(moduleId, settings) {
                    root.groupCommandSettings[moduleId] = settings
                    let temp = root._commandList
                    root._commandList = []
                    root._commandList = temp
                }
            }
        }
    }

    function refreshModel() {
        // Settings can now be populated by parent instead of this component.
        let connectorList = Object.keys(root.groupConnectorSettings)
        connectorList.sort()
        root._connectorList = connectorList
        if (root._connectorList.length === 0) {
            refreshConnectorList()
        }

        let monitorList = Object.keys(root.groupMonitorSettings)
        monitorList.sort()
        root._monitorList = monitorList
        if (root._monitorList.length === 0) {
            refreshMonitorList()
        }

        let commandList = Object.keys(root.groupCommandSettings)
        commandList.sort()
        root._commandList = commandList
        if (root._commandList.length === 0) {
            refreshCommandList()
        }
    }

    function refreshConnectorList() {
        root._connectorList = []
        root.groupConnectorSettings = {}

        let connectorIds = LK.config.getGroupModuleIds(root.groupName, "connector")
        for (let connectorId of connectorIds) {
            root.groupConnectorSettings[connectorId] = LK.config.getGroupModuleSettings(root.groupName, connectorId).map(JSON.parse)
        }

        // Set last since this controls when list is re-rendered.
        root._connectorList = connectorIds
    }

    function refreshMonitorList() {
        root._monitorList = []
        root.groupMonitorSettings = {}

        let monitorIds = LK.config.getGroupModuleIds(root.groupName, "monitor")
        for (let monitorId of monitorIds) {
            root.groupMonitorSettings[monitorId] = LK.config.getGroupModuleSettings(root.groupName, monitorId).map(JSON.parse)
        }

        // Set last since this controls when list is re-rendered.
        root._monitorList = monitorIds
    }

    function refreshCommandList() {
        root._commandList = []
        root.groupCommandSettings = {}

        let commandIds = LK.config.getGroupModuleIds(root.groupName, "command")
        for (let commandId of commandIds) {
            root.groupCommandSettings[commandId] = LK.config.getGroupModuleSettings(root.groupName, commandId).map(JSON.parse)
        }

        // Set last since this controls when list is re-rendered.
        root._commandList = commandIds
    }

    function resetModel() {
        root.groupName = ""
        root._connectorList = []
        root._monitorList = []
        root._commandList = []
        root.groupConnectorSettings = {}
        root.groupMonitorSettings = {}
        root.groupCommandSettings = {}
        root._loading = true
    }
}
