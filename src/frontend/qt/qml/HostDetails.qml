import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "js/TextTransform.js" as TextTransform

Item {
    id: root
    required property var model
    property var hostData: model.get_host_data(model.selected_row)
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
            model: root.hostData.length > 0 ?
                groupByCategory(root.model.get_monitor_data(root.hostData[1]), root.model.get_command_data(root.hostData[1])) :
                0
 
            GroupBox {
                title: modelData.category
                Layout.minimumWidth: 0.5 * root.columnMaximumWidth
                Layout.maximumWidth: root.columnMaximumWidth
                Layout.alignment: Qt.AlignTop

                ColumnLayout {
                    anchors.top: parent.top
                    implicitWidth: parent.width

                    Repeater {
                        model: modelData.monitors

                        PropertyRow {
                            label: modelData.display_options.display_name
                            value: modelData.values[0].value + " " + modelData.display_options.unit
                        }
                    }
                    Repeater {
                        model: modelData.commands

                        PropertyRow {
                            label: modelData.display_options.display_name
                            value: modelData.results[0].message
                        }
                    }
                }
            }
        }
    }

    function groupByCategory(monitorDataJsons, commandDataJsons) {
        let categories = []
        let monitorsCategorized = {}
        let commandsCategorized = {}

        monitorDataJsons.forEach(json => {
            let data = JSON.parse(json)
            let category = data.display_options.category

            if (!categories.includes(category)) {
                categories.push(category)
                monitorsCategorized[category] = [ data ]
            }
            else {
                monitorsCategorized[category].push(data)
            }
        })

        commandDataJsons.forEach(json => {
            let data = JSON.parse(json)
            let category = data.display_options.category

            if (!categories.includes(category)) {
                categories.push(category)
            }
            if (category in commandsCategorized) {
                commandsCategorized[category].push(data)
            }
            else {
                commandsCategorized[category] = [ data ]
            }
        })

        // Essentially a list of key-value pairs.
        return categories.map(category => ({
            category: TextTransform.capitalize(category),
            monitors: monitorsCategorized[category],
            commands: commandsCategorized[category]
        }))
    }

}