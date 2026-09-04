/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Button"
import "../Text"
import ".."
import "../StyleOverride"

/// One module-type section (connectors / monitors / commands) with list, add, and edit.
ColumnLayout {
    id: root
    spacing: Theme.spacingTight

    property string title: ""
    property string emptyPlaceholder: "No changes"
    property var moduleIds: []
    property var settingsByModule: ({})
    property bool readOnly: false
    property bool allowAddRemove: !root.readOnly
    property bool showSudoBadge: false
    property int buttonSize: 26
    property string groupName: ""
    property string hostId: ""
    property string moduleType: ""
    property string addDialogLabel: ""
    /// When true, settingsByModule already holds full setting definitions (no group lookup).
    property bool settingsAreComplete: false

    signal removeModule(string moduleId)
    signal addModule(string moduleId)
    signal updateModuleSettings(string moduleId, var settings)

    RowLayout {
        Layout.fillWidth: true

        BigText {
            topPadding: Theme.spacingLoose
            text: root.title
            Layout.fillWidth: true
        }

        ImageButton {
            visible: root.allowAddRemove
            imageSource: "qrc:/main/images/button/add"
            onClicked: {
                let candidates = LK.config.getUnselectedModuleIds(root.moduleIds, root.moduleType)
                addDialog.inputSpecs = [{
                    label: root.addDialogLabel,
                    field_type: "Option",
                    options: candidates,
                    option_descriptions: candidates.map((id) => LK.config.getModuleDescription(id))
                }]
                addDialog.open()
            }
            flatButton: true
            roundButton: false
            tooltip: "Add new module"
            size: root.buttonSize

            Layout.alignment: Qt.AlignBottom
        }
    }

    OptionalText {
        visible: root.moduleIds.length === 0
        placeholder: root.emptyPlaceholder
        text: ""

        Layout.leftMargin: Theme.commonIndent
    }

    Repeater {
        model: root.moduleIds

        Column {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.commonIndent

            RowHighlight {
                width: parent.width
                height: moduleRow.height

                onClicked: {
                    descriptionText.text = descriptionText.text !== "" ? "" : LK.config.getModuleDescription(modelData)
                }

                Column {
                    id: moduleRow
                    width: parent.width

                    RowLayout {
                        width: parent.width
                        spacing: Theme.spacingTight

                        NormalText {
                            text: modelData
                            Layout.alignment: Qt.AlignVCenter
                        }

                        PillText {
                            visible: root.showSudoBadge && LK.config.moduleRequiresSudo(modelData)
                            text: "sudo"
                            pillColor: Theme.colorForCriticality("Info")
                            tooltip: "may require sudo for root privileges"
                            Layout.alignment: Qt.AlignVCenter
                            Layout.leftMargin: Theme.spacingLoose
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        ImageButton {
                            visible: !root.readOnly
                            enabled: root.settingsByModule[modelData].length > 0
                            imageSource: "qrc:/main/images/button/entry-edit"
                            onClicked: {
                                editDialog.moduleId = modelData
                                let full = root.settingsAreComplete
                                    ? root.settingsByModule[modelData]
                                    : LK.config.getGroupModuleSettings(root.groupName, modelData).map(JSON.parse)
                                editDialog.moduleSettings =
                                    root.mergeModuleSettingsWithCache(full, root.settingsByModule[modelData])
                                editDialog.open()
                            }
                            flatButton: true
                            roundButton: false
                            tooltip: "Module settings..."
                            size: root.buttonSize

                            Layout.alignment: Qt.AlignVCenter
                        }

                        ImageButton {
                            visible: root.allowAddRemove
                            imageSource: "qrc:/main/images/button/delete"
                            onClicked: root.removeModule(modelData)
                            flatButton: true
                            roundButton: false
                            tooltip: "Remove module from group"
                            size: root.buttonSize

                            Layout.alignment: Qt.AlignVCenter
                        }
                    }

                    SmallText {
                        id: descriptionText
                        visible: text !== ""
                        opacity: visible ? 1 : 0
                        text: ""
                        color: Theme.textColorDark
                    }
                }
            }

            Repeater {
                model: root.settingsByModule[modelData].filter((setting) => setting.enabled === true)

                RowLayout {
                    SmallText {
                        text: modelData.key + ": "
                        color: Theme.textColorDark

                        Layout.leftMargin: Theme.commonIndent
                    }

                    SmallText {
                        text: modelData.isSecret === true ? "••••••" : modelData.value
                        color: Theme.textColorDark

                        Layout.fillWidth: true
                    }
                }
            }
        }
    }

    InputDialog {
        id: addDialog
        width: 500
        height: 200
        inputSpecs: [{
            label: root.addDialogLabel,
            field_type: "Option",
            options: [],
            option_descriptions: []
        }]
        onInputValuesGiven: function(inputValues) {
            root.addModule(inputValues[0])
        }
    }

    ModuleSettingsDialog {
        id: editDialog
        groupName: root.groupName
        hostId: root.hostId

        onSettingsUpdated: function(moduleId, moduleSettings) {
            root.updateModuleSettings(moduleId, moduleSettings)
        }
    }

    // TODO: move implementation to config-backend?
    function mergeModuleSettingsWithCache(fullSettings, cached) {
        if (!cached || cached.length !== fullSettings.length) {
            return fullSettings
        }

        for (let i = 0; i < fullSettings.length; i++) {
            let found = cached.find(item => item.key === fullSettings[i].key)
            if (found) {
                fullSettings[i].value = found.value
                fullSettings[i].enabled = found.enabled
                fullSettings[i].secretBackend = LK.config.detectSecretBackend(found.value)
            }
        }
        return fullSettings
    }
}
