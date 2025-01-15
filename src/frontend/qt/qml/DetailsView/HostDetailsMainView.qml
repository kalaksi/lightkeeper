import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0

import ".."
import "../Misc"
import "../Text"
import "../js/Parse.js" as Parse

Item {
    id: root
    property string hostId: ""
    property var _hostDetails: Parse.TryParseJson(LK.hosts.getHostDataJson(hostId))
    property var _categories: []
    property bool _showEmptyCategories: true

    property int columnMinimumWidth: Theme.groupboxMinWidth
    property int columnMaximumWidth: Theme.groupboxMaxWidth
    property int columnMinimumHeight: 450
    property int columnMaximumHeight: 450
    property int columnSpacing: Theme.spacingNormal

    // Link between invocation and command button has to be stored and handled on higher level and not in
    // e.g. CommandButton or CommandButtonRow since those are not persistent.
    property var _invocationIdToButton: {}

    signal customCommandsDialogOpened()

    Component.onCompleted: {
        root._invocationIdToButton = {}
        root._categories = []
        root.refresh()
    }

    Connections {
        target: LK.hosts

        function onMonitoringDataReceived(hostId, category, monitoringDataQv, invocationId) {
            if (hostId === root.hostId) {
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

            HostGroupBox {
                id: hostGroupBox
                visible: root._categories.includes("host")
                categoryName: "host"
                // TODO: fix warnings when using esc to close details view.
                hostId: root._hostDetails.host.name
                status: root._hostDetails.status
                fqdn: root._hostDetails.host.fqdn
                ipAddress: root._hostDetails.host.ip_address

                Layout.minimumWidth: root.columnMinimumWidth
                Layout.maximumWidth: root.columnMaximumWidth
                Layout.preferredWidth: root.columnMinimumWidth +
                                        (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) - root.columnSpacing
                Layout.minimumHeight: root.columnMinimumHeight
                Layout.maximumHeight: root.columnMaximumHeight
                Layout.alignment: Qt.AlignTop

                onRefreshClicked: {
                    LK.command.refreshMonitorsOfCategory(root.hostId, hostGroupBox.categoryName)
                    hostGroupBox.refreshProgress = 0
                }

            }

            CustomCommandGroupBox {
                visible: false
                Layout.minimumWidth: root.columnMinimumWidth
                Layout.maximumWidth: root.columnMaximumWidth
                Layout.preferredWidth: root.columnMinimumWidth +
                                        (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) - root.columnSpacing
                Layout.minimumHeight: root.columnMinimumHeight
                Layout.maximumHeight: root.columnMaximumHeight
                Layout.alignment: Qt.AlignTop

                onConfigClicked: {
                    root.customCommandsDialogOpened()
                }
            }

            Repeater {
                model: root._categories.filter(category => category !== "host")

                CategoryGroupBox {
                    id: groupBox
                    categoryName: modelData

                    Layout.minimumWidth: root.columnMinimumWidth
                    Layout.maximumWidth: root.columnMaximumWidth
                    Layout.preferredWidth: root.columnMinimumWidth +
                                            (rootScrollView.availableWidth % root.columnMinimumWidth / grid.columns) - root.columnSpacing
                    Layout.minimumHeight: root.columnMinimumHeight
                    Layout.maximumHeight: root.columnMaximumHeight
                    Layout.alignment: Qt.AlignTop

                    onRefreshClicked: {
                        LK.command.refreshMonitorsOfCategory(root.hostId, groupBox.categoryName)
                        groupBox.refreshProgress = 0
                    }

                    Connections {
                        target: LK.hosts

                        // Update group box monitor refresh progress and remove block/mask if finished.
                        function onMonitoringDataReceived(hostId, category, monitoringDataQv) {
                            if (hostId === root.hostId && category === groupBox.categoryName) {
                                groupBox.refreshProgress = LK.hosts.getPendingMonitorCountForCategory(root.hostId, category) > 0 ?  0 : 100

                                if (monitoringDataQv !== undefined) {
                                    propertyTable.model.update(monitoringDataQv)
                                }
                            }
                        }

                        // Update command progress. Starts automatic refresh of relevant monitors if finished.
                        function onCommandResultReceived(commandResultJson, invocationId) {
                            if (root._invocationIdToButton[invocationId] !== undefined) {
                                let buttonId = root._invocationIdToButton[invocationId]
                                let progress = LK.hosts.getPendingCommandProgress(invocationId)
                                categoryCommands.updateProgress(buttonId, progress)
                                propertyTable.updateProgress(buttonId, progress)

                                if (progress >= 100) {
                                    let commandResult = JSON.parse(commandResultJson)
                                    LK.command.refreshMonitorsOfCommand(root.hostId, commandResult.command_id)
                                    delete root._invocationIdToButton[invocationId]
                                }
                            }
                        }
                    }

                    Connections {
                        target: LK.command

                        function onHostInitializing(hostId) {
                            if (hostId === root.hostId) {
                                groupBox.refreshProgress = 0
                            }
                        }
                        // Reset command progress to 0.
                        function onCommandExecuted(invocationId, hostId, commandId, category, buttonId) {
                            if (hostId === root.hostId && category === groupBox.categoryName) {
                                root._invocationIdToButton[invocationId] = buttonId

                                categoryCommands.updateProgress(buttonId, 0)
                                propertyTable.updateProgress(buttonId, 0)
                            }
                        }
                    }

                    ColumnLayout {
                        id: column
                        anchors.fill: parent
                        spacing: 0

                        // Category-level command buttons (buttons on top of the category area).
                        CommandButtonRow {
                            id: categoryCommands
                            visible: commands.length > 0
                            size: 34
                            flatButtons: false
                            roundButtons: false
                            commands: Parse.ListOfJsons(LK.command.getCommandsOnLevel(root.hostId, modelData, "", 0))
                            hoverEnabled: !groupBox.blocked

                            Layout.alignment: Qt.AlignHCenter
                            Layout.bottomMargin: Theme.spacingLoose

                            onClicked: function(buttonId, commandId, params) {
                                LK.command.execute(buttonId, root.hostId, commandId, params)
                            }
                        }

                        PropertyTable {
                            id: propertyTable
                            category: modelData
                            monitoring_datas: LK.hosts.getCategoryMonitorIds(root.hostId, modelData)
                                                      .map(monitorId => LK.hosts.getMonitoringData(root.hostId, monitorId))
                            command_datas: LK.command.getCategoryCommands(root.hostId, modelData)

                            Layout.fillHeight: true
                            Layout.fillWidth: true

                            onButtonClicked: function(buttonId, commandId, params) {
                                LK.command.execute(buttonId, root.hostId, commandId, params)
                            }
                        }
                    }
                }
            }
        }
    }

    function refresh() {
        if (root.hostId !== "") {
            root._hostDetails = Parse.TryParseJson(LK.hosts.getHostDataJson(hostId))
            // TODO: effect on performance if checking categories every time?
            root._categories =  LK.hosts.getCategories(root.hostId, !root._showEmptyCategories)
                                        .map(category_qv => category_qv.toString())
        }
    }

    function refreshContent() {
        if (root.hostId !== "") {
            LK.command.forceInitializeHost(hostId)
        }
    }

    function activate() {
        // Do nothing.
    }

    function deactivate() {
        // Do nothing.
    }
}
