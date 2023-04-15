import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import ".."
import "../Text"
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
    property var _categories: getCategories()
    // Contains invocation IDs. Keeps track of monitoring data refresh progress. Empty when all is done.
    property var _pendingMonitorInvocations: {}
    property var _maximumPendingInvocations: {}


    onHostIdChanged: {
        root._categories = getCategories()
    }

    Component.onCompleted: {
        root._pendingMonitorInvocations = {}
        root._maximumPendingInvocations = {}
    }

    Connections {
        target: HostDataManager
        function onMonitoring_data_received(invocation_id, category, monitoring_data_qv) {
            // Refresh list of categories.
            root._categories = getCategories()
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
                model: root._categories

                GroupBox {
                    id: groupBox
                    leftPadding: Theme.groupbox_padding()
                    rightPadding: Theme.groupbox_padding()
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
                        id: groupBoxLabel
                        anchors.left: groupBox.left
                        anchors.right: groupBox.right

                        text: TextTransform.capitalize(modelData)
                        icon: Theme.category_icon(modelData)
                        color: Theme.category_color(modelData)
                        onRefreshClicked: function() {
                            let invocation_ids = CommandHandler.refresh_monitors_of_category(root.hostId, modelData)
                            if (invocation_ids.length > 0) {
                                root._pendingMonitorInvocations[modelData] = invocation_ids
                                root._maximumPendingInvocations[modelData] = invocation_ids.length
                                groupBoxLabel.refreshProgress = 0.0
                            }
                        }

                        Connections {
                            target: HostDataManager
                            function onMonitoring_data_received(invocation_id, category, monitoring_data_qv) {
                                // Keep track of ongoing monitor invocations.
                                // TODO: move these to HostDataManager? Easier to track pending invocations that come from elsewhere.
                                if (category === modelData &&
                                    root._pendingMonitorInvocations[category] !== undefined &&
                                    root._maximumPendingInvocations[category] !== undefined) {

                                    let index = root._pendingMonitorInvocations[category].indexOf(invocation_id)
                                    if (index >= 0) {
                                        // Remove from array of pending monitor invocations.
                                        root._pendingMonitorInvocations[category].splice(index, 1)
                                    }

                                    if (root._maximumPendingInvocations[category] > 0) {
                                        groupBoxLabel.refreshProgress = 1.0 - root._pendingMonitorInvocations[category].length /
                                                                              root._maximumPendingInvocations[category]
                                    }
                                    else {
                                        groupBoxLabel.refreshProgress = 1.0
                                    }
                                }
                            }
                        }
                    }

                    ColumnLayout {
                        id: column
                        anchors.fill: parent
                        spacing: 0

                        // Category-level command buttons (buttons on top of the category area).
                        CommandButtonRow {
                            visible: commands.length > 0
                            size: 34
                            flatButtons: false
                            roundButtons: false
                            commands: Parse.ListOfJsons(CommandHandler.get_commands_on_level(root.hostId, modelData, "", 0))

                            Layout.alignment: Qt.AlignHCenter
                            Layout.topMargin: size * 0.20
                            Layout.bottomMargin: size * 0.30

                            onClicked: function(commandId, params) {
                                let invocationId = CommandHandler.execute(root.hostId, commandId, params)
                            }
                        }

                        // Host details are a bit different from monitor data, so handling it separately here.
                        Item {
                            id: hostDetails
                            visible: modelData === "host"
                            width: parent.width
                            height: 90

                            // Background.
                            Rectangle {
                                x: -(groupBox.width - column.width) / 2
                                width: groupBox.width
                                height: parent.height
                                color: "#50808080"
                            }

                            Row {
                                anchors.fill: parent
                                spacing: 20
                                leftPadding: 20

                                Image {
                                    id: hostIcon
                                    anchors.verticalCenter: parent.verticalCenter
                                    source: "qrc:/main/images/host/linux"
                                    width: 60
                                    height: 60
                                }

                                Column {
                                    anchors.verticalCenter: parent.verticalCenter

                                    Repeater {
                                        width: 0.7 * parent.width
                                        model: root._hostDetails !== null ?
                                            [
                                                [ "Status", root._hostDetails.status ],
                                                [ "Name", root._hostDetails.name ],
                                                [ "FQDN", root._hostDetails.domain_name ],
                                                [ "IP Address", root._hostDetails.ip_address ],
                                            ] : []

                                        Row {
                                            visible: modelData[1] !== ""
                                            width: column.width
                                            rightPadding: 0.1 * column.width
                                            spacing: 0.075 * column.width

                                            Label {
                                                width: 0.25 * parent.width
                                                verticalAlignment: Text.AlignVCenter
                                                lineHeight: 0.6
                                                text: modelData[0]
                                            }

                                            SmallText {
                                                width: 0.35 * parent.width
                                                text: modelData[1]
                                            }
                                        }
                                    }
                                }
                            }
                        }


                        PropertyTable {
                            id: propertyTable
                            // Default to 10 just to avoid warnings of zero length
                            // width: parent.width > 0 ? parent.width : 10
                            hostId: root.hostId
                            category: modelData
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

            if (root.hideEmptyCategories) {
                categories = categories.filter(category => !HostDataManager.is_empty_category(root.hostId, category))
            }

            return categories
        }
        return []
    }

    function refresh() {
        root._hostDetailsJson = HostDataManager.get_host_data_json(hostId)
        root._hostDetails = Parse.TryParseJson(_hostDetailsJson)
    }
}
