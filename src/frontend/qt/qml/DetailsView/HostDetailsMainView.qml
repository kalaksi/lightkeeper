import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import ".."
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse

Item {
    id: root
    property string hostId: ""
    property bool hideEmptyCategories: true
    property int columnMinimumWidth: 400
    property int columnMaximumWidth: 600
    property int columnMinimumHeight: 450
    property int columnMaximumHeight: 450
    property int columnSpacing: 6
    property var _hostDetailsJson: HostDataManager.get_host_data_json(hostId)
    property var _hostDetails: Parse.TryParseJson(_hostDetailsJson)
    // Contains invocation IDs. Keeps track of monitoring data refresh progress. Empty when all is done.
    property var _pendingMonitorInvocations: []
    property int _maximumPendingInvocations: 0


    Connections {
        target: HostDataManager
        function onMonitoring_data_received(invocation_id, category, monitoring_data_qv) {
            // Remove from array of pending monitor invocations.
            let index = root._pendingMonitorInvocations.indexOf(invocation_id)
            if (index >= 0) {
                root._pendingMonitorInvocations.splice(index, 1)
            }
            if (root._pendingMonitorInvocations.length === 0) {
                root._maximumPendingInvocations = 0
            }
        }
    }

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
                model: getCategories()

                GroupBox {
                    id: groupBox
                    leftPadding: 5
                    rightPadding: 5
                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.preferredWidth: root.columnMinimumWidth +
                                           (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) -
                                           root.columnSpacing
                    Layout.minimumHeight: root.columnMinimumHeight
                    Layout.maximumHeight: root.columnMaximumHeight
                    Layout.alignment: Qt.AlignTop

                    background: Rectangle {
                        color: Theme.category_background_color()
                    }

                    // Custom label provides more flexibility.
                    label: GroupBoxLabel {
                        anchors.left: groupBox.left
                        anchors.right: groupBox.right

                        text: TextTransform.capitalize(modelData)
                        icon: Theme.category_icon(modelData)
                        color: Theme.category_color(modelData)
                        refreshProgress: 1.0 - root._pendingMonitorInvocations.length / root._maximumPendingInvocations
                        onRefreshClicked: function() {
                            let invocation_ids = CommandHandler.refresh_monitors_of_category(root.hostId, modelData)
                            if (invocation_ids.length > 0) {
                                root._pendingMonitorInvocations.push(...invocation_ids)
                                root._maximumPendingInvocations = root._pendingMonitorInvocations.length
                            }
                        }
                    }

                    ColumnLayout {
                        anchors.fill: parent

                        // Category-level command buttons (buttons on top of the category area).
                        CommandButtonRow {
                            // anchors.horizontalCenter: parent.horizontalCenter
                            size: 34
                            flatButtons: false
                            roundButtons: false
                            commands: Parse.ListOfJsons(CommandHandler.get_child_commands(root.hostId, modelData, "", 0))
                            onClicked: (commandId) => CommandHandler.execute(root.hostId, commandId, [""])
                        }

                        // Host data is a bit different from monitor data, so handling it separately here.
                        /*
                        Repeater {
                            model: modelData === "host" && root._hostDetails !== null ?
                                [
                                    [ "Status", root._hostDetails.status ],
                                    [ "Name", root._hostDetails.name ],
                                    [ "FQDN", root._hostDetails.domain_name ],
                                    [ "IP Address", root._hostDetails.ip_address ],
                                ] : []

                            // TODO: get rid
                            PropertyRow {
                                label: modelData[0]
                                value: modelData[1]
                            }
                        }
                        */


                        PropertyTable {
                            id: propertyTable
                            // Default to 10 just to avoid warnings of zero length
                            width: parent.width > 0 ? parent.width : 10
                            hostId: root.hostId
                            monitoring_datas: HostDataManager.get_category_monitor_ids(root.hostId, modelData)
                                                             .map(monitorId => HostDataManager.get_monitoring_data(root.hostId, monitorId))
                            command_datas: CommandHandler.get_category_commands(root.hostId, modelData)

                            Connections {
                                target: HostDataManager
                                function onMonitoring_data_received(invocation_id, category, monitoring_data_qv) {
                                    if (category === modelData) {
                                        propertyTable.model.update(monitoring_data_qv)
                                    }
                                }
                            }

                            Layout.fillHeight: true
                            Layout.fillWidth: true
                        }
                    }
                }
            }
        }
    }

    function getCategories() {
        if (root.hostId !== "") {
            let categories = HostDataManager.get_categories(root.hostId)
                                            .map(category_qv => category_qv.toString())
            return categories
        }
        return []
    }

    function refresh() {
        root._hostDetailsJson = HostDataManager.get_host_data_json(hostId)
        root._hostDetails = Parse.TryParseJson(_hostDetailsJson)
    }
}
