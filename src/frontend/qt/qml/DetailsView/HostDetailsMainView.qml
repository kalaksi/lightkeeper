import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import "../StyleOverride"
import ".."
import "../Misc"
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
    property var _hostDetailsJson: LK.hosts.getHostDataJson(hostId)
    property var _hostDetails: Parse.TryParseJson(_hostDetailsJson)
    property var _categories: {}
    property bool _showEmptyCategories: true


    Component.onCompleted: {
        root._categories = []
        refresh()
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

            Repeater {
                model: root._categories

                GroupBox {
                    id: groupBox
                    property bool blocked: true
                    property var _invocationIdToButton: {}

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

                    Component.onCompleted: {
                        groupBox._invocationIdToButton = {}
                    }

                    Connections {
                        target: LK.command

                        function onCommandExecuted(invocationId, hostId, commandId, category, buttonId) {
                            if (hostId === root.hostId && category === modelData) {
                                // State has to be stored and handled on higher level and not in e.g.
                                // CommandButton or CommandButtonRow since those are not persistent.
                                groupBox._invocationIdToButton[invocationId] = buttonId

                                categoryCommands.updateProgress(buttonId, 0)
                                propertyTable.updateProgress(buttonId, 0)
                            }
                        }
                    }

                    Connections {
                        target: LK.hosts

                        function onCommandResultReceived(commandResultJson, invocationId) {
                            if (groupBox._invocationIdToButton[invocationId] !== undefined) {
                                let buttonId = groupBox._invocationIdToButton[invocationId]
                                let progress = LK.hosts.getPendingCommandProgress(invocationId)
                                categoryCommands.updateProgress(buttonId, progress)
                                propertyTable.updateProgress(buttonId, progress)

                                if (progress >= 100) {
                                    let commandResult = JSON.parse(commandResultJson)
                                    LK.command.refreshMonitorsOfCommand(root.hostId, commandResult.command_id)
                                    delete groupBox._invocationIdToButton[invocationId]
                                }
                            }
                        }
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
                            LK.command.refreshMonitorsOfCategory(root.hostId, modelData)
                            groupBoxLabel.refreshProgress = 0
                            groupBox.blocked = true
                        }

                        Connections {
                            target: LK.hosts
                            function onMonitoringDataReceived(hostId, category, monitoring_data_qv) {
                                if (hostId === root.hostId && category === modelData) {
                                    groupBoxLabel.refreshProgress = LK.hosts.getPendingMonitorCountForCategory(root.hostId, category) > 0 ?  0 : 100

                                    if (groupBoxLabel.refreshProgress >= 100) {
                                        groupBox.blocked = false
                                    }
                                }
                            }
                        }

                        Connections {
                            target: LK.command
                            function onHostInitializing(hostId) {
                                if (hostId === root.hostId) {
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
                            id: categoryCommands
                            visible: commands.length > 0
                            size: 34
                            flatButtons: false
                            roundButtons: false
                            commands: Parse.ListOfJsons(LK.command.getCommandsOnLevel(root.hostId, modelData, "", 0))
                            hoverEnabled: !groupBox.blocked

                            Layout.alignment: Qt.AlignHCenter
                            Layout.topMargin: size * 0.20
                            Layout.bottomMargin: size * 0.30

                            onClicked: function(buttonId, commandId, params) {
                                LK.command.execute(buttonId, root.hostId, commandId, params)
                            }
                        }

                        // Host details are a bit different from other monitor data, so handling it separately here.
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
                            category: modelData
                            monitoring_datas: LK.hosts.getCategoryMonitorIds(root.hostId, modelData)
                                                             .map(monitorId => LK.hosts.getMonitoringData(root.hostId, monitorId))
                            command_datas: LK.command.getCategoryCommands(root.hostId, modelData)

                            Layout.fillHeight: true
                            Layout.fillWidth: true

                            onButtonClicked: function(buttonId, commandId, params) {
                                LK.command.execute(buttonId, root.hostId, commandId, params)
                            }

                            Connections {
                                target: LK.hosts
                                function onMonitoringDataReceived(hostId, category, monitoringDataQv) {
                                    if (hostId === root.hostId && category === modelData) {
                                        propertyTable.model.update(monitoringDataQv)
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
            root._hostDetailsJson = LK.hosts.getHostDataJson(hostId)
            root._hostDetails = Parse.TryParseJson(_hostDetailsJson)
            root._categories =  LK.hosts.getCategories(root.hostId, !root._showEmptyCategories)
                                               .map(category_qv => category_qv.toString())
        }
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
