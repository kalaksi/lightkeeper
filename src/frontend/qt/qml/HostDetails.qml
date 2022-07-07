import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "js/TextTransform.js" as TextTransform
import "js/Parse.js" as Parse

Item {
    id: root
    required property var model
    required property var commandsModel
    property var hostData: model.get_host_data(model.selected_row)
    // For convenience
    property string hostId: root.hostData.length > 0 ? root.hostData[1] : ""
    property int columnMaximumWidth: 500

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    GridLayout {
        id: grid
        anchors.fill: parent
        columns: 6
        rows: 2

        GroupBox {
            title: "Host"
            Layout.minimumWidth: 0.5 * root.columnMaximumWidth
            Layout.maximumWidth: root.columnMaximumWidth
            Layout.alignment: Qt.AlignTop

            ColumnLayout {
                id: column
                anchors.top: parent.top
                width: parent.width

                // TODO: get rid of the manual indexing and length checking
                PropertyRow {
                    label: "Status"
                    value: root.hostData.length > 0 ? root.hostData[0] : ""
                }

                PropertyRow {
                    label: "Name"
                    value: root.hostData.length > 0 ? root.hostData[1] : ""
                }

                PropertyRow {
                    label: "FQDN"
                    value: root.hostData.length > 0 ? root.hostData[2] : ""
                }

                PropertyRow {
                    label: "IP address"
                    value: root.hostData.length > 0 ? root.hostData[3] : ""
                }
            }

        }
 
        Repeater {
            model: root.hostId !== "" ?
                groupByCategory(root.model.get_monitor_data_map(root.hostId), root.commandsModel.get_commands(root.hostId)) :
                []
 
            GroupBox {
                title: modelData.category
                Layout.minimumWidth: 0.5 * root.columnMaximumWidth
                Layout.maximumWidth: root.columnMaximumWidth
                Layout.alignment: Qt.AlignTop

                ColumnLayout {
                    anchors.top: parent.top
                    implicitWidth: parent.width

                    // Category-level command buttons.
                    Repeater {
                        model: root.commandsModel.get_child_commands(root.hostId, "")

                        CommandButton {
                            property var parsedData: JSON.parse(modelData)

                            icon_source: "qrc:/main/images/button/refresh"
                            onClicked: root.commandsModel.execute(root.hostId, parsedData.command_id)
                        }
                    }

                    Repeater {
                        model: modelData.monitorDatas

                        PropertyRow {
                            label: modelData.display_options.display_name
                            value: modelData.values[0].value + " " + modelData.display_options.unit
                            childCommands: Parse.ListOfJsons(root.commandsModel.get_child_commands(root.hostId, modelData.monitor_id))
                        }
 
                    }
                }
            }
        }
    }

    function groupByCategory(monitorDataJsons, commandJsons) {
        // TODO: calculate categories on rust side (HostData).
        let categories = []
        let monitorDataByCategory = {}
        let commandByCategory = {}

        for (let monitorId in monitorDataJsons) {
            let data = JSON.parse(monitorDataJsons[monitorId])
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

}