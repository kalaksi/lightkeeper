import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import ".."
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse
import "../js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property var hostDataManager
    property string hostId: ""
    property bool hideEmptyCategories: true
    property int columnMinimumWidth: 400
    property int columnMaximumWidth: 600
    property int columnMinimumHeight: 450
    property int columnMaximumHeight: 450
    property int columnSpacing: 6
    property var _hostData: groupByCategory(root.hostDataManager.get_monitor_datas(hostId), CommandHandler.get_commands(root.hostId))

    ScrollView {
        id: rootScrollView
        anchors.fill: parent
        contentWidth: availableWidth

        GridLayout {
            id: grid
            anchors.fill: parent
            columns: Math.floor(parent.width / root.columnMinimumWidth)
            columnSpacing: root.columnSpacing

            Repeater {
                // TODO: hide empty categories
                //model: root.hideEmptyCategories ?
                //    root._hostData.filter((item) => item.monitorDatas !== undefined && item.monitorDatas.filter((data) => data.criticality !== "Ignore").length > 0) :
                //    root._hostData
                model: {
                    // console.log(JSON.stringify(root._hostData))
                    return root._hostData
                }

                GroupBox {
                    id: groupBox
                    property bool _hasOnlyMultivalues: modelData.monitorDatas.filter(item => !item.display_options.use_multivalue).length === 0
                    leftPadding: 2
                    rightPadding: 2
                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.preferredWidth: root.columnMinimumWidth +
                                           (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) -
                                           root.columnSpacing
                    Layout.minimumHeight: root.columnMinimumHeight
                    Layout.maximumHeight: root.columnMaximumHeight
                    Layout.alignment: Qt.AlignTop

                    background: Rectangle {
                        color: "#404040"
                    }

                    // Custom label provides more flexibility.
                    label: GroupBoxLabel {
                        id: customLabel
                        text: modelData.category
                        anchors.left: groupBox.left
                        anchors.right: groupBox.right
                    }

                    ScrollView {
                        anchors.fill: parent
                        anchors.topMargin: customLabel.height
                        contentWidth: availableWidth

                        Column {
                            anchors.left: parent.left
                            anchors.right: parent.right

                            // Category-level command buttons (buttons on top of the category area).
                            CommandButtonRow {
                                anchors.horizontalCenter: parent.horizontalCenter
                                size: 34
                                flatButtons: false
                                roundButtons: false
                                commands: Parse.ListOfJsons(CommandHandler.get_child_commands(root.hostId, modelData.category, "", 0))
                                onClicked: function(commandId) {
                                    CommandHandler.execute(root.hostId, commandId, [""])
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

                            // Go through monitoring datas and create rows.
                            Repeater {
                                model: modelData.monitorDatas.filter((item) => item.criticality !== "Ignore")

                                Column {
                                    property var monitorData: modelData
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    spacing: root.columnSpacing

                                    // Header text for multivalues.
                                    Label {
                                        width: parent.width
                                        padding: 5
                                        topPadding: 10
                                        horizontalAlignment: Text.AlignHCenter
                                        text: monitorData.display_options.display_text
                                        visible: monitorData.display_options.use_multivalue && !groupBox._hasOnlyMultivalues

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
                                        model: getPropertyRows(monitorData)

                                        PropertyRow {
                                            label: monitorData.display_options.use_multivalue ? modelData.label : monitorData.display_options.display_text
                                            value: ValueUnit.AsText(modelData.value, monitorData.display_options.unit)
                                            criticality: modelData.criticality.toLowerCase()
                                            displayStyle: monitorData.display_options.display_style

                                            hostId: root.hostId
                                            commandParams: modelData.command_params
                                            rowCommands: Parse.ListOfJsons(
                                                CommandHandler.get_child_commands(
                                                    root.hostId, monitorData.display_options.category, monitorData.monitor_id, modelData.multivalue_level
                                                )
                                            )
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

    // Practically flattens multivalue data and does some filtering.
    function getPropertyRows(monitorData) {
        let lastDataPoint = monitorData.values.slice(-1)[0]
        let result = []
        if (monitorData.display_options.use_multivalue) {

            lastDataPoint.multivalue.forEach(multivalue => {
                multivalue.multivalue_level = 1
                result.push(multivalue)

                // 2nd level of multivalues.
                multivalue.multivalue.forEach(multivalue2 => {
                    // Add indent for 2nd level values.
                    multivalue2.label = "    " + multivalue2.label
                    multivalue2.multivalue_level = 2
                    result.push(multivalue2)
                })

                // Now remove the duplicate data.
                multivalue.multivalue = []
            })
        }
        else {
            lastDataPoint.multivalue_level = 0
            result = [ lastDataPoint ]
        }
        return result.filter(item => item.criticality !== "Ignore")
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
        root._hostData = groupByCategory(root.hostDataManager.get_monitor_datas(hostId), CommandHandler.get_commands(root.hostId))
    }

}
