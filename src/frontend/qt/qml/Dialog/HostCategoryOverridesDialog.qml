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
    property string hostId: ""
    property string categoryName: ""
    property var monitorSettings: ({})
    property var commandSettings: ({})
    property var monitorEnabled: ({})
    property var commandEnabled: ({})
    property var _monitorList: []
    property var _commandList: []
    property bool _loading: true
    property int _buttonSize: 26

    title: `Host overrides for ${root.categoryName}`
    implicitWidth: 630
    implicitHeight: 700
    standardButtons: Dialog.Cancel | Dialog.Ok
    signal configurationChanged()

    onOpened: {
        LK.config.beginHostConfiguration()
        root.refreshModel()
        root._loading = false
    }

    onAccepted: {
        LK.config.updateHostCategoryModuleSettings(
            root.hostId,
            JSON.stringify(root.packModuleConfigs(root.monitorSettings, root.monitorEnabled)),
            JSON.stringify(root.packModuleConfigs(root.commandSettings, root.commandEnabled)))
        LK.config.endHostConfiguration()
        root.configurationChanged()
        root.resetModel()
    }

    onRejected: {
        LK.config.cancelHostConfiguration()
        root.resetModel()
    }

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
            visible: !root._loading
            anchors.fill: parent
            anchors.rightMargin: Theme.marginScrollbar
            spacing: Theme.spacingTight

            ModuleConfigSection {
                title: "Enabled monitoring modules and settings"
                emptyPlaceholder: "No modules in this category"
                moduleIds: root._monitorList
                settingsByModule: root.monitorSettings
                enabledByModule: root.monitorEnabled
                allowAddRemove: false
                allowDisable: true
                showSudoBadge: true
                hostId: root.hostId
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onUpdateModuleSettings: function(moduleId, settings) {
                    root.monitorSettings[moduleId] = settings
                    root.refreshMonitorList()
                }

                onSetModuleEnabled: function(moduleId, enabled) {
                    root.monitorEnabled[moduleId] = enabled
                    root.refreshMonitorList()
                }
            }

            ModuleConfigSection {
                title: "Enabled command modules and settings"
                emptyPlaceholder: "No modules in this category"
                moduleIds: root._commandList
                settingsByModule: root.commandSettings
                enabledByModule: root.commandEnabled
                allowAddRemove: false
                allowDisable: true
                showSudoBadge: true
                hostId: root.hostId
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onUpdateModuleSettings: function(moduleId, settings) {
                    root.commandSettings[moduleId] = settings
                    root.refreshCommandList()
                }

                onSetModuleEnabled: function(moduleId, enabled) {
                    root.commandEnabled[moduleId] = enabled
                    root.refreshCommandList()
                }
            }
        }
    }

    function refreshModel() {
        let monitors = JSON.parse(
            LK.config.getHostCategoryModuleSettings(root.hostId, root.categoryName, "monitor"))
        root.monitorSettings = {}
        root.monitorEnabled = {}
        for (let moduleId of Object.keys(monitors)) {
            root.monitorSettings[moduleId] = monitors[moduleId].settings
            root.monitorEnabled[moduleId] = monitors[moduleId].enabled
        }
        root.refreshMonitorList()

        let commands = JSON.parse(
            LK.config.getHostCategoryModuleSettings(root.hostId, root.categoryName, "command"))
        root.commandSettings = {}
        root.commandEnabled = {}
        for (let moduleId of Object.keys(commands)) {
            root.commandSettings[moduleId] = commands[moduleId].settings
            root.commandEnabled[moduleId] = commands[moduleId].enabled
        }
        root.refreshCommandList()
    }

    function refreshMonitorList() {
        let monitorIds = Object.keys(root.monitorSettings)
        monitorIds.sort()
        root._monitorList = []
        root._monitorList = monitorIds
    }

    function refreshCommandList() {
        let commandIds = Object.keys(root.commandSettings)
        commandIds.sort()
        root._commandList = []
        root._commandList = commandIds
    }

    function packModuleConfigs(settingsByModule, enabledByModule) {
        let packed = {}
        for (let moduleId of Object.keys(settingsByModule)) {
            packed[moduleId] = {
                enabled: enabledByModule[moduleId] !== false,
                settings: settingsByModule[moduleId],
            }
        }
        return packed
    }

    function resetModel() {
        root.hostId = ""
        root.categoryName = ""
        root._monitorList = []
        root._commandList = []
        root.monitorSettings = {}
        root.commandSettings = {}
        root.monitorEnabled = {}
        root.commandEnabled = {}
        root._loading = true
    }
}
