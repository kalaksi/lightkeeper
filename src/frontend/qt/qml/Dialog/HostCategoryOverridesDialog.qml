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
    property var _monitorList: []
    property var _commandList: []
    property bool _loading: true
    property int _buttonSize: 26

    title: `Host overrides: ${root.categoryName}`
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
            JSON.stringify(root.monitorSettings),
            JSON.stringify(root.commandSettings))
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
                allowAddRemove: false
                showSudoBadge: true
                hostId: root.hostId
                settingsAreComplete: true
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onUpdateModuleSettings: function(moduleId, settings) {
                    root.monitorSettings[moduleId] = settings
                    let temp = root._monitorList
                    root._monitorList = []
                    root._monitorList = temp
                }
            }

            ModuleConfigSection {
                title: "Enabled command modules and settings"
                emptyPlaceholder: "No modules in this category"
                moduleIds: root._commandList
                settingsByModule: root.commandSettings
                allowAddRemove: false
                showSudoBadge: true
                hostId: root.hostId
                settingsAreComplete: true
                buttonSize: root._buttonSize
                Layout.fillWidth: true

                onUpdateModuleSettings: function(moduleId, settings) {
                    root.commandSettings[moduleId] = settings
                    let temp = root._commandList
                    root._commandList = []
                    root._commandList = temp
                }
            }
        }
    }

    function refreshModel() {
        root.monitorSettings = JSON.parse(
            LK.config.getHostCategoryModuleSettings(root.hostId, root.categoryName, "monitor"))
        let monitorIds = Object.keys(root.monitorSettings)
        monitorIds.sort()
        root._monitorList = monitorIds

        root.commandSettings = JSON.parse(
            LK.config.getHostCategoryModuleSettings(root.hostId, root.categoryName, "command"))
        let commandIds = Object.keys(root.commandSettings)
        commandIds.sort()
        root._commandList = commandIds
    }

    function resetModel() {
        root.hostId = ""
        root.categoryName = ""
        root._monitorList = []
        root._commandList = []
        root.monitorSettings = {}
        root.commandSettings = {}
        root._loading = true
    }
}
