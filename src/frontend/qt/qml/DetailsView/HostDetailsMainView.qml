import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import "../StyleOverride"
import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse

Item {
    id: root
    property string hostId: ""
    property int columnMinimumWidth: Theme.groupboxMinWidth
    property int columnMaximumWidth: Theme.groupboxMaxWidth
    property int columnMinimumHeight: 450
    property int columnMaximumHeight: 450
    property int columnSpacing: Theme.spacingNormal
    property var _hostDetailsJson: HostDataManager.get_host_data_json(hostId)
    property var _hostDetails: Parse.TryParseJson(_hostDetailsJson)
    property var _categories: {}
    property bool _showEmptyCategories: true


    Component.onCompleted: {
        refresh()
    }

    Connections {
        target: HostDataManager

        function onMonitoring_data_received(host_id, category, monitoring_data_qv) {
            if (host_id === root.hostId) {
                root.refresh()
            }
        }
    }

    // ScrollView doesn't have boundsBehavior so this is the workaround.
    Binding {
        target: rootScrollView.contentItem
        property: "boundsBehavior"
        value: Flickable.StopAtBounds
    }

    WorkingSprite {
        visible: root._categories.length === 0
        scale: 1.5
        text: "Connecting..."
    }

    ScrollView {
        id: rootScrollView
        anchors.fill: parent
        contentWidth: availableWidth
        clip: true

        GridLayout {
            id: grid
            columns: Math.floor(parent.width / root.columnMinimumWidth)
            columnSpacing: root.columnSpacing

            Repeater {
                model: root._categories

                GroupBox {
                    id: groupBox
                    property bool blocked: true

                    leftPadding: Theme.spacingTight
                    rightPadding: Theme.spacingTight
                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.preferredWidth: root.columnMinimumWidth +
                                           (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) -
                                           root.columnSpacing
                    Layout.minimumHeight: root.columnMinimumHeight
                    Layout.maximumHeight: root.columnMaximumHeight
                    Layout.alignment: Qt.AlignTop

                    background: Rectangle {
                        color: Theme.categoryBackgroundColor
                    }

                    // Custom label provides more flexibility.
                    label: GroupBoxLabel {
                        id: groupBoxLabel
                        anchors.left: groupBox.left
                        anchors.right: groupBox.right

                        text: TextTransform.capitalize(modelData)
                        icon: Theme.categoryIcon(modelData)
                        color: Theme.categoryColor(modelData)
                        onRefreshClicked: function() {
                            CommandHandler.force_refresh_monitors_of_category(root.hostId, modelData)
                            groupBoxLabel.refreshProgress = 0
                            groupBox.blocked = true
                        }

                        Connections {
                            target: HostDataManager
                            function onMonitoring_data_received(host_id, category, monitoring_data_qv) {
                                if (host_id === root.hostId && category === modelData) {
                                    groupBoxLabel.refreshProgress = HostDataManager.getPendingMonitorCountForCategory(root.hostId, category) > 0 ?  0 : 100

                                    if (isCategoryReady(category)) {
                                        groupBox.blocked = false
                                    }
                                }
                            }
                        }

                        Connections {
                            target: CommandHandler
                            function onHost_initializing(host_id) {
                                if (host_id === root.hostId) {
                                    groupBoxLabel.refreshProgress = 0
                                    groupBox.blocked = true
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
                            hoverEnabled: !groupBox.blocked

                            Layout.alignment: Qt.AlignHCenter
                            Layout.topMargin: size * 0.20
                            Layout.bottomMargin: size * 0.30

                            onClicked: function(commandId, params) {
                                CommandHandler.execute(root.hostId, commandId, params)
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
                                    sourceSize.width: 64
                                    sourceSize.height: 64
                                }

                                Column {
                                    anchors.verticalCenter: parent.verticalCenter

                                    Repeater {
                                        width: 0.7 * parent.width
                                        model: root._hostDetails !== null ?
                                            [
                                                [ "Name", root._hostDetails.host.name ],
                                                [ "Status", root._hostDetails.status ],
                                                [ "FQDN", root._hostDetails.host.fqdn ],
                                                [ "IP Address", root._hostDetails.host.ip_address ],
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
                            hostId: root.hostId
                            category: modelData
                            monitoring_datas: HostDataManager.get_category_monitor_ids(root.hostId, modelData)
                                                             .map(monitorId => HostDataManager.get_monitoring_data(root.hostId, monitorId))
                            command_datas: CommandHandler.get_category_commands(root.hostId, modelData)

                            Layout.fillHeight: true
                            Layout.fillWidth: true
                            Layout.minimumWidth: 50

                            Connections {
                                target: HostDataManager
                                function onMonitoring_data_received(host_id, category, monitoring_data_qv) {
                                    if (host_id === root.hostId && category === modelData) {
                                        propertyTable.model.update(monitoring_data_qv)
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        anchors.fill: parent
                        color: Theme.categoryRefreshMask
                        visible: groupBox.blocked

                        MouseArea {
                            anchors.fill: parent
                            preventStealing: true
                        }
                    }
                }
            }
        }
    }

    function refresh() {
        if (root.hostId !== "") {
            root._hostDetailsJson = HostDataManager.get_host_data_json(hostId)
            root._hostDetails = Parse.TryParseJson(_hostDetailsJson)
            root._categories =  HostDataManager.getCategories(root.hostId, !root._showEmptyCategories)
                                               .map(category_qv => category_qv.toString())
        }
    }

    function isCategoryReady(category) {
        return HostDataManager.is_host_initialized(root.hostId) &&
               HostDataManager.getPendingMonitorCountForCategory(root.hostId, category) == 0
    }

    function focus() {
        // Do nothing.
    }

    function unfocus() {
        // Do nothing.
    }

    function close() {
        // Do nothing.
    }
}
