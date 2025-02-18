import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../Button"
import "../Text"
import ".."
import "../StyleOverride"

LightkeeperDialog {
    id: root
    required property string groupName 
    property var _connectorList: []
    property var _groupConnectorSettings: {}
    property var _monitorList: []
    property var _groupMonitorSettings: {}
    property var _commandList: []
    property var _groupCommandSettings: {}
    property bool _loading: true
    property int _buttonSize: 26

    title: `Configuration group: ${root.groupName}`
    implicitWidth: 600
    implicitHeight: 670
    standardButtons: Dialog.Cancel | Dialog.Ok

    Component.onCompleted: {
        resetModel()
    }

    onOpened: {
        root.refreshModel()
        root._loading = false
    }

    onAccepted: {
        let connectorSettingsJson = JSON.stringify(root._groupConnectorSettings)
        let monitorSettingsJson = JSON.stringify(root._groupMonitorSettings)
        let commandSettingsJson = JSON.stringify(root._groupCommandSettings)

        LK.config.updateGroupModules(root.groupName, connectorSettingsJson, monitorSettingsJson, commandSettingsJson)
        LK.config.writeGroupConfiguration()
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

            RowLayout {
                width: parent.width

                BigText {
                    topPadding: Theme.spacingLoose
                    text: "Connector module settings"

                    Layout.fillWidth: true
                }

                ImageButton {
                    imageSource: "qrc:/main/images/button/add"
                    onClicked: {
                        let connectors = LK.config.getUnselectedConnectorIds(root._connectorList)
                        connectorAddDialog.inputSpecs = [{
                            label: "Connector module",
                            field_type: "Option",
                            options: connectors,
                            option_descriptions: connectors.map((connector) => LK.config.getConnectorDescription(connector))
                        }]
                        connectorAddDialog.open()
                    }
                    flatButton: true
                    roundButton: false
                    tooltip: "Add new module"
                    size: root._buttonSize

                    Layout.alignment: Qt.AlignBottom
                }
            }

            OptionalText {
                visible: root._connectorList.length === 0
                placeholder: "No changes"
                text: ""

                Layout.leftMargin: Theme.commonIndent
            }


            Repeater {
                id: connectorRepeater
                model: root._connectorList

                Column {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.commonIndent

                    RowHighlight {
                        id: connectorHighlighter
                        width: parent.width
                        height: connectorModuleRow.height

                        onClicked: {
                            connectorDescriptionText.text = connectorDescriptionText.text !== "" ? "" : LK.config.getConnectorDescription(modelData)
                        }

                        Column {
                            id: connectorModuleRow
                            width: parent.width

                            RowLayout {
                                width: parent.width
                                spacing: Theme.spacingTight

                                NormalText {
                                    text: modelData
                                    Layout.alignment: Qt.AlignVCenter
                                }

                                // Spacer
                                Item {
                                    Layout.fillWidth: true
                                }

                                ImageButton {
                                    enabled: {
                                        root._groupConnectorSettings[modelData].length > 0
                                    }
                                    imageSource: "qrc:/main/images/button/entry-edit"
                                    onClicked: {
                                        connectorEditDialog.moduleId = modelData
                                        connectorEditDialog.moduleSettings = root._groupConnectorSettings[modelData]
                                        connectorEditDialog.open()
                                    }
                                    flatButton: true
                                    roundButton: false
                                    tooltip: "Module settings..."
                                    size: root._buttonSize

                                    Layout.alignment: Qt.AlignVCenter
                                }

                                ImageButton {
                                    imageSource: "qrc:/main/images/button/delete"
                                    onClicked: {
                                        root.removeConnector(modelData)
                                    }
                                    flatButton: true
                                    roundButton: false
                                    tooltip: "Remove module from group"
                                    size: root._buttonSize

                                    Layout.alignment: Qt.AlignVCenter
                                }
                            }

                            SmallText {
                                id: connectorDescriptionText
                                visible: text !== ""
                                opacity: visible ? 1 : 0
                                text: ""
                                color: Theme.textColorDark
                            }
                        }
                    }

                    Repeater {
                        model: root._groupConnectorSettings[modelData].filter((setting) => setting.enabled === true)

                        RowLayout {
                            SmallText {
                                text: modelData.key + ": "
                                color: Theme.textColorDark

                                Layout.leftMargin: Theme.commonIndent
                            }

                            SmallText {
                                text: modelData.value
                                color: Theme.textColorDark

                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }

            RowLayout {
                width: parent.width

                BigText {
                    topPadding: Theme.spacingLoose
                    text: "Enabled monitoring modules and settings"

                    Layout.fillWidth: true
                }

                ImageButton {
                    imageSource: "qrc:/main/images/button/add"
                    onClicked: {
                        let monitors = LK.config.getUnselectedMonitorIds(root._monitorList)
                        monitorAddDialog.inputSpecs = [{
                            label: "Monitoring module",
                            field_type: "Option",
                            options: monitors,
                            option_descriptions: monitors.map((monitor) => LK.config.getMonitorDescription(monitor))
                        }]
                        monitorAddDialog.open()
                    }
                    flatButton: true
                    roundButton: false
                    tooltip: "Add new module"
                    size: root._buttonSize

                    Layout.alignment: Qt.AlignBottom
                }
            }

            OptionalText {
                visible: monitorRepeater.model.length === 0
                placeholder: "No changes"
                text: ""

                Layout.leftMargin: Theme.commonIndent
            }

            Repeater {
                id: monitorRepeater
                model: root._monitorList

                Column {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.commonIndent

                    RowHighlight {
                        id: monitorHighlighter
                        width: parent.width
                        height: monitoringModuleRow.height

                        onClicked: {
                            monitorDescriptionText.text = monitorDescriptionText.text !== "" ? "" : LK.config.getMonitorDescription(modelData)
                        }

                        Column {
                            id: monitoringModuleRow
                            width: parent.width

                            RowLayout {
                                width: parent.width
                                spacing: Theme.spacingTight

                                NormalText {
                                    text: modelData
                                    Layout.alignment: Qt.AlignVCenter
                                }

                                // Spacer
                                Item {
                                    Layout.fillWidth: true
                                }

                                ImageButton {
                                    enabled: {
                                        root._groupMonitorSettings[modelData].length > 0
                                    }
                                    imageSource: "qrc:/main/images/button/entry-edit"
                                    onClicked: {
                                        monitorEditDialog.moduleId = modelData
                                        monitorEditDialog.moduleSettings = root._groupMonitorSettings[modelData]
                                        monitorEditDialog.open()
                                    }
                                    flatButton: true
                                    roundButton: false
                                    tooltip: "Module settings..."
                                    size: root._buttonSize

                                    Layout.alignment: Qt.AlignVCenter
                                }

                                ImageButton {
                                    imageSource: "qrc:/main/images/button/delete"
                                    onClicked: {
                                        root.removeMonitor(modelData)
                                    }
                                    flatButton: true
                                    roundButton: false
                                    tooltip: "Remove module from group"
                                    size: root._buttonSize

                                    Layout.alignment: Qt.AlignVCenter
                                }
                            }
                            SmallText {
                                id: monitorDescriptionText
                                visible: text !== ""
                                opacity: visible ? 1 : 0
                                text: ""
                                color: Theme.textColorDark
                            }
                        }
                    }

                    Repeater {
                        model: root._groupMonitorSettings[modelData].filter((setting) => setting.enabled === true)

                        RowLayout {
                            SmallText {
                                text: modelData.key + ": "
                                color: Theme.textColorDark

                                Layout.leftMargin: Theme.commonIndent
                            }

                            SmallText {
                                text: modelData.value
                                color: Theme.textColorDark

                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }

            RowLayout {
                width: parent.width

                BigText {
                    topPadding: Theme.spacingLoose
                    text: "Enabled command modules and settings"

                    Layout.fillWidth: true
                }

                ImageButton {
                    imageSource: "qrc:/main/images/button/add"
                    onClicked: {
                        let commands = LK.config.getUnselectedCommandIds(root._commandList)
                        commandAddDialog.inputSpecs = [{
                            label: "Command module",
                            field_type: "Option",
                            options: commands,
                            option_descriptions: commands.map((command) => LK.config.getCommandDescription(command))
                        }]
                        commandAddDialog.open()
                    }
                    flatButton: true
                    roundButton: false
                    tooltip: "Add new module"
                    size: root._buttonSize

                    Layout.alignment: Qt.AlignBottom
                }
            }

            OptionalText {
                visible: commandRepeater.model.length === 0
                placeholder: "No changes"
                text: ""

                Layout.leftMargin: Theme.commonIndent
            }

            Repeater {
                id: commandRepeater
                model: root._commandList

                Column {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.commonIndent

                    RowHighlight {
                        id: commandHighlighter
                        width: parent.width
                        height: commandModuleRow.height

                        onClicked: {
                            commandDescriptionText.text = commandDescriptionText.text !== "" ? "" : LK.config.getCommandDescription(modelData)
                        }

                        Column {
                            id: commandModuleRow
                            width: parent.width

                            RowLayout {
                                width: parent.width
                                spacing: Theme.spacingTight

                                NormalText {
                                    text: modelData

                                    Layout.alignment: Qt.AlignVCenter
                                    Layout.rightMargin: Theme.spacingNormal
                                }

                                // Spacer
                                Item {
                                    Layout.fillWidth: true
                                }

                                ImageButton {
                                    enabled: {
                                        root._groupCommandSettings[modelData].length > 0
                                    }
                                    imageSource: "qrc:/main/images/button/entry-edit"
                                    onClicked: {
                                        commandEditDialog.moduleId = modelData
                                        commandEditDialog.moduleSettings = root._groupCommandSettings[modelData]
                                        commandEditDialog.open()
                                    }
                                    flatButton: true
                                    roundButton: false
                                    tooltip: "Module settings..."
                                    size: root._buttonSize

                                    Layout.alignment: Qt.AlignVCenter
                                }

                                ImageButton {
                                    imageSource: "qrc:/main/images/button/delete"
                                    onClicked: {
                                        root.removeCommand(modelData)
                                    }
                                    flatButton: true
                                    roundButton: false
                                    tooltip: "Remove module from group"
                                    size: root._buttonSize

                                    Layout.alignment: Qt.AlignVCenter
                                }
                            }

                            SmallText {
                                id: commandDescriptionText
                                visible: text !== ""
                                text: ""
                                color: Theme.textColorDark
                            }
                        }
                    }

                    Repeater {
                        model: root._groupCommandSettings[modelData].filter((setting) => setting.enabled === true)

                        RowLayout {
                            SmallText {
                                text: modelData.key + ": "
                                color: Theme.textColorDark

                                Layout.leftMargin: Theme.commonIndent
                            }

                            SmallText {
                                text: modelData.value
                                color: Theme.textColorDark

                                Layout.fillWidth: true
                            }
                        }
                    }
                }
            }
        }
    }

    InputDialog {
        id: connectorAddDialog
        width: parent.width
        height: 200
        inputSpecs: [{
            label: "Connector module",
            field_type: "Option",
            options: {},
            option_descriptions: {}
        }]
        onInputValuesGiven: function(inputValues) {
            root._groupConnectorSettings[inputValues[0]] = LK.config.getGroupConnectorSettings(root.groupName, inputValues[0]).map(JSON.parse)
            let newConnectors = root._connectorList.concat(inputValues[0])
            newConnectors.sort()
            root._connectorList = newConnectors
        }
    }

    ModuleSettingsDialog {
        id: connectorEditDialog
        anchors.centerIn: undefined

        onSettingsUpdated: function(moduleId, moduleSettings) {
            root._groupConnectorSettings[moduleId] = moduleSettings
            // Re-render the list.
            root._connectorList = root._connectorList.slice()
        }
    }

    InputDialog {
        id: monitorAddDialog
        width: parent.width
        height: 200
        inputSpecs: [{
            label: "Monitoring module",
            field_type: "Option",
            options: {},
            option_descriptions: {}
        }]
        onInputValuesGiven: function(inputValues) {
            root._groupMonitorSettings[inputValues[0]] = LK.config.getGroupMonitorSettings(root.groupName, inputValues[0]).map(JSON.parse)
            let newMonitors = root._monitorList.concat(inputValues[0])
            newMonitors.sort()
            root._monitorList = newMonitors
        }
    }

    ModuleSettingsDialog {
        id: monitorEditDialog
        anchors.centerIn: undefined

        onSettingsUpdated: function(moduleId, moduleSettings) {
            root._groupMonitorSettings[moduleId] = moduleSettings
            // Re-render the list.
            root._monitorList = root._monitorList.slice()
        }
    }

    InputDialog {
        id: commandAddDialog
        width: parent.width
        height: 200
        inputSpecs: [{
            label: "Command module",
            field_type: "Option",
            options: {},
            option_descriptions: {}
        }]
        onInputValuesGiven: function(inputValues) {
            root._groupCommandSettings[inputValues[0]] = LK.config.getGroupCommandSettings(root.groupName, inputValues[0]).map(JSON.parse)
            let newCommands = root._commandList.concat(inputValues[0])
            newCommands.sort()
            root._commandList = newCommands
        }
    }

    ModuleSettingsDialog {
        id: commandEditDialog
        anchors.centerIn: undefined

        onSettingsUpdated: function(moduleId, moduleSettings) {
            root._groupCommandSettings[moduleId] = moduleSettings
            // Re-render the list.
            root._commandList = root._commandList.slice()
        }
    }

    function refreshModel() {
        refreshConnectorList()
        refreshMonitorList()
        refreshCommandList()
    }

    function refreshConnectorList() {
        root._connectorList = []
        root._groupConnectorSettings = {}

        let connectorIds = LK.config.getGroupConnectorIds(root.groupName)
        for (let connectorId of connectorIds) {
            root._groupConnectorSettings[connectorId] = LK.config.getGroupConnectorSettings(root.groupName, connectorId).map(JSON.parse)
        }

        // Set last since this controls when list is re-rendered.
        root._connectorList = connectorIds
    }

    function refreshMonitorList() {
        root._monitorList = []
        root._groupMonitorSettings = {}

        let monitorIds = LK.config.getGroupMonitorIds(root.groupName)
        for (let monitorId of monitorIds) {
            root._groupMonitorSettings[monitorId] = LK.config.getGroupMonitorSettings(root.groupName, monitorId).map(JSON.parse)
        }

        // Set last since this controls when list is re-rendered.
        root._monitorList = monitorIds
    }

    function refreshCommandList() {
        root._commandList = []
        root._groupCommandSettings = {}

        let commandIds = LK.config.getGroupCommandIds(root.groupName)
        for (let commandId of commandIds) {
            root._groupCommandSettings[commandId] = LK.config.getGroupCommandSettings(root.groupName, commandId).map(JSON.parse)
        }

        // Set last since this controls when list is re-rendered.
        root._commandList = commandIds
    }

    function removeConnector(moduleId) {
        delete root._groupConnectorSettings[moduleId]
        root._connectorList = root._connectorList.filter((connector) => connector !== moduleId)
    }

    function removeMonitor(moduleId) {
        delete root._groupMonitorSettings[moduleId]
        root._monitorList = root._monitorList.filter((monitor) => monitor !== moduleId)
    }

    function removeCommand(moduleId) {
        delete root._groupCommandSettings[moduleId]
        root._commandList = root._commandList.filter((command) => command !== moduleId)
    }

    function resetModel() {
        root._loading = true
        root._connectorList = []
        root._groupConnectorSettings = {}
        root._monitorList = []
        root._groupMonitorSettings = {}
        root._commandList = []
        root._groupCommandSettings = {}
    }
}