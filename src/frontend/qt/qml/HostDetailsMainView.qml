import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "js/TextTransform.js" as TextTransform
import "js/Parse.js" as Parse
import "js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property var commandHandler
    required property var hostDataManager
    property string hostId: ""
    property int columnMinimumWidth: 400
    property int columnMaximumWidth: 400
    property int columnMaximumHeight: 400
    property var _hostData: groupByCategory(root.hostDataManager.get_monitor_data_map(hostId), root.commandHandler.get_commands(root.hostId))

    ScrollView {
        anchors.fill: parent
        contentWidth: availableWidth

        GridLayout {
            id: grid
            anchors.fill: parent
            columns: Math.floor(parent.width / root.columnMinimumWidth)

            Repeater {
                model: root._hostData

                GroupBox {
                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.maximumHeight: root.columnMaximumHeight
                    Layout.alignment: Qt.AlignTop

                    label: Label {
                        width: parent.width
                        padding: 5
                        horizontalAlignment: Text.AlignHCenter
                        text: modelData.category

                        background: Rectangle{
                            anchors.fill: parent
                            gradient: Gradient {
                                GradientStop { position: 0.0; color: "#606060" }
                                GradientStop { position: 1.0; color: "#404040" }
                            }
                        }
                    }

                    background: Rectangle {
                        color: "#404040"
                    }

                    ScrollView {
                        anchors.fill: parent
                        contentWidth: availableWidth

                        Column {
                            // anchors.fill: parent
                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            spacing: 5

                            // Category-level command buttons (buttons on top of the category area).
                            /*
                            CommandButtonRow {
                                commands: Parse.ListOfJsons(root.commandHandler.get_child_commands(root.hostId, ""))
                                onClicked: function(targetId) {
                                    root.commandHandler.execute(root.hostId, modelData.command_id, targetId)
                                }
                            }
                            */

                            // Host data is a bit different from monitor data, so handling it separately here.
                            Repeater {
                                model: modelData.category === "Host" ? Object.entries(root.hostDataManager.get_host_data(root.hostId)) : []

                                PropertyRow {
                                    label: modelData[0]
                                    value: modelData[1]
                                }
                            }

                            Repeater {
                                model: modelData.monitorDatas

                                // Creates multiple rows for multivalue-entries, otherwise just one row.
                                Repeater {
                                    id: rowRepeater
                                    property var monitorData: modelData
                                    property var lastDataPoint: modelData.values.slice(-1)[0]
                                    model: modelData.display_options.use_multivalue ? lastDataPoint.multivalue : [ lastDataPoint ]

                                    PropertyRow {
                                        label: modelData.label.length > 0 ? modelData.label : monitorData.display_options.display_text
                                        value: ValueUnit.AsText(modelData.value, rowRepeater.monitorData.display_options.unit)

                                        hostId: root.hostId
                                        targetId: modelData.source_id
                                        rowCommands: Parse.ListOfJsons(root.commandHandler.get_child_commands(root.hostId, monitorData.monitor_id))
                                        commandHandler: root.commandHandler
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    function groupByCategory(monitorDataJsons, commandJsons) {
        let categories = []
        let monitorDataByCategory = {}
        let commandByCategory = {}

        for (let monitorId in monitorDataJsons) {
            let data = JSON.parse(monitorDataJsons[monitorId])
            // TODO: could be done better than to add a property ad-hoc?
            data.monitor_id = monitorId

            let category = data.display_options.category
            categories.push(category)

            if (category in monitorDataByCategory) {
                monitorDataByCategory[category].push(data)
            }
            else {
                monitorDataByCategory[category] = [ data ]
            }
        }

        commandJsons.forEach(json => {
            let data = JSON.parse(json)
            let category = data.display_options.category
            categories.push(category)

            if (category in commandByCategory) {
                commandByCategory[category].push(data)
            }
            else {
                commandByCategory[category] = [ data ]
            }
        })

        let uniqueCategories = [...new Set(categories)]
        return uniqueCategories.map(category => ({
            category: TextTransform.capitalize(category),
            monitorDatas: monitorDataByCategory[category] || [],
            commands: commandByCategory[category] || [],
        }))
    }

    function refresh() {
        root._hostData = groupByCategory(root.hostDataManager.get_monitor_data_map(hostId), root.commandHandler.get_commands(root.hostId))
    }

}