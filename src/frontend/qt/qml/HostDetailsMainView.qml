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
    property int columnMinimumHeight: 400
    property int columnMaximumHeight: 400
    property int rowSpacing: 5
    property var _hostData: groupByCategory(root.hostDataManager.get_monitor_datas(hostId), root.commandHandler.get_commands(root.hostId))

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
                    id: box
                    property bool _hasOnlyMultivalues: modelData.monitorDatas.filter(item => !item.display_options.use_multivalue).length === 0
                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.minimumHeight: root.columnMinimumHeight
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
                            anchors.left: parent.left
                            anchors.right: parent.right
                            spacing: root.rowSpacing

                            // Category-level command buttons (buttons on top of the category area).
                            CommandButtonRow {
                                anchors.horizontalCenter: parent.horizontalCenter
                                size: 34
                                flatButtons: false
                                roundButtons: false
                                commands: Parse.ListOfJsons(root.commandHandler.get_child_commands(root.hostId, modelData.category, ""))
                                onClicked: function(commandId) {
                                    root.commandHandler.execute(root.hostId, commandId, [""])
                                }
                            }

                            // Host data is a bit different from monitor data, so handling it separately here.
                            Repeater {
                                model: modelData.category === "Host" ? Object.entries(root.hostDataManager.get_host_data(root.hostId)) : []

                                PropertyRow {
                                    label: modelData[0]
                                    value: modelData[1]
                                }
                            }

                            Repeater {
                                model: modelData.monitorDatas.filter((item) => item.criticality !== "Ignore")

                                Column {
                                    property var monitorData: modelData
                                    property var lastDataPoint: modelData.values.slice(-1)[0]
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    spacing: root.rowSpacing

                                    // Header text for multivalues.
                                    Label {
                                        width: parent.width
                                        padding: 5
                                        topPadding: 10
                                        horizontalAlignment: Text.AlignHCenter
                                        text: monitorData.display_options.display_text
                                        visible: monitorData.display_options.use_multivalue && !box._hasOnlyMultivalues

                                        background: Rectangle {
                                            width: parent.width
                                            height: 2
                                            anchors.bottom: parent.bottom
                                            gradient: Gradient {
                                                orientation: Gradient.Horizontal
                                                GradientStop { position: 0.0; color: "#404040" }
                                                GradientStop { position: 0.5; color: "#555555" }
                                                GradientStop { position: 1.0; color: "#404040" }
                                            }
                                        }
                                    }

                                    // Creates multiple rows for multivalue-entries, otherwise just one row.
                                    Repeater {
                                        id: rowRepeater
                                        property var monitorData: parent.monitorData
                                        property var lastDataPoint: parent.lastDataPoint
                                        model: monitorData.display_options.use_multivalue ?
                                                    lastDataPoint.multivalue.filter((item) => item.criticality !== "Ignore") :
                                                    [ lastDataPoint ]

                                        PropertyRow {
                                            label: monitorData.display_options.use_multivalue ? modelData.label : monitorData.display_options.display_text
                                            value: ValueUnit.AsText(modelData.value, monitorData.display_options.unit)
                                            useProgressBar: monitorData.display_options.display_style === "ProgressBar"

                                            hostId: root.hostId
                                            targetId: modelData.source_id
                                            rowCommands: Parse.ListOfJsons(
                                                root.commandHandler.get_child_commands(root.hostId, monitorData.display_options.category, monitorData.monitor_id)
                                            )
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
    }

    function groupByCategory(monitorDataJsons, commandJsons) {
        let categories = []
        let monitorDataByCategory = {}
        let commandByCategory = {}

        monitorDataJsons.forEach(json => {
            let data = JSON.parse(json)
            let category = data.display_options.category
            categories.push(category)

            if (category in monitorDataByCategory) {
                monitorDataByCategory[category].push(data)
            }
            else {
                monitorDataByCategory[category] = [ data ]
            }
        })

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
        root._hostData = groupByCategory(root.hostDataManager.get_monitor_datas(hostId), root.commandHandler.get_commands(root.hostId))
    }

}
