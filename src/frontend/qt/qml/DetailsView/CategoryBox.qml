import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15


Item {
    id: root
    property var monitorDatas: {}
    property string backgroundColor: ""
    property bool _hasOnlyMultivalues: monitorDatas.filter(item => !item.display_options.use_multivalue).length === 0

    GroupBox {
        anchors.fill: parent
        leftPadding: 2
        rightPadding: 2

        background: Rectangle {
            color: root.backgroundColor
        }

        // Custom label provides more flexibility.
        label: GroupBoxLabel {
            anchors.left: root.left
            anchors.right: root.right

            text: modelData.category
            icon: Theme.category_icon(modelData.category)
            color: Theme.category_color(modelData.category)
            onRefreshClicked: function() {
                let invocation_ids = CommandHandler.refresh_monitors_of_category(root.hostId, modelData.category)
                root._pendingMonitorInvocations.push(...invocation_ids)
            }
        }

        ScrollView {
            anchors.fill: parent
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
                    onClicked: (commandId) => CommandHandler.execute(root.hostId, commandId, [""])
                }

                // Host data is a bit different from monitor data, so handling it separately here.
                Repeater {
                    model: modelData.category === "Host" && root._hostDetails !== null ?
                        [
                            [ "Status", root._hostDetails.status ],
                            [ "Name", root._hostDetails.name ],
                            [ "FQDN", root._hostDetails.domain_name ],
                            [ "IP Address", root._hostDetails.ip_address ],
                        ] : []

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
                            visible: monitorData.display_options.use_multivalue && !root._hasOnlyMultivalues

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