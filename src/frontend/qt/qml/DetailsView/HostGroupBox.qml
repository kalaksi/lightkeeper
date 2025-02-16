import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../Text"
import "../js/Parse.js" as Parse
import "../StyleOverride"


CategoryGroupBox {
    id: root

    property string hostId: ""
    property string status: ""
    property string fqdn: ""
    property string ipAddress: ""
    property string _categoryName: "host"
    property int refreshProgress: 100
    property bool blocked: refreshProgress < 100

    Connections {
        target: LK.hosts

        // Update group box monitor refresh progress and remove block/mask if finished.
        function onMonitoringDataReceived(hostId, category, monitoringDataQv) {
            if (hostId === root.hostId && category === root.categoryName) {
                root.refreshProgress = LK.hosts.getPendingMonitorCountForCategory(root.hostId, category) > 0 ?  0 : 100
                propertyTable.model.update(monitoringDataQv)
            }
        }
    }

    Connections {
        target: LK.command

        function onHostInitializing(hostId) {
            if (hostId === root.hostId) {
                root.refreshProgress = 0
            }
        }
        // Reset command progress to 0.
        function onCommandExecuted(invocationId, hostId, commandId, category, buttonId) {
            if (hostId === root.hostId && category === root.categoryName) {
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
            commands: LK.command.getCommandsOnLevel(root.hostId, root._categoryName, "", 0).map(JSON.parse)
            hoverEnabled: !root.blocked

            Layout.alignment: Qt.AlignHCenter
            Layout.bottomMargin: Theme.spacingLoose

            onClicked: function(buttonId, commandId, params) {
                LK.command.execute(buttonId, root.hostId, commandId, params)
            }
        }

        Item {
            id: hostDetails
            width: parent.width
            height: 90

            // Background.
            Rectangle {
                x: -(root.width - column.width) / 2
                width: root.width
                height: parent.height
                color: "#50808080"
            }

            Row {
                spacing: 20
                topPadding: 10
                leftPadding: 20
                rightPadding: 0.1 * width

                Image {
                    id: hostIcon
                    anchors.verticalCenter: parent.verticalCenter
                    source: "qrc:/main/images/host/linux"
                    sourceSize.width: 64
                    sourceSize.height: 64
                }

                Column {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 0.7 * parent.width

                    Row {
                        width: parent.width
                        spacing: 0.075 * parent.width

                        Label {
                            width: 0.25 * parent.width
                            lineHeight: 0.6
                            text: "Name"
                        }

                        SmallText {
                            verticalAlignment: Text.AlignVCenter
                            width: 0.35 * parent.width
                            text: root.hostId
                        }
                    }

                    Row {
                        width: parent.width
                        spacing: 0.075 * parent.width

                        Label {
                            width: 0.25 * parent.width
                            lineHeight: 0.6
                            text: "Status"
                        }

                        SmallText {
                            verticalAlignment: Text.AlignVCenter
                            width: 0.35 * parent.width
                            text: root.status
                        }
                    }

                    Row {
                        width: parent.width
                        spacing: 0.075 * parent.width

                        Label {
                            width: 0.25 * parent.width
                            lineHeight: 0.6
                            text: "FQDN"
                        }

                        SmallText {
                            verticalAlignment: Text.AlignVCenter
                            width: 0.35 * parent.width
                            text: root.fqdn
                        }
                    }

                    Row {
                        width: parent.width
                        spacing: 0.075 * parent.width

                        Label {
                            width: 0.25 * parent.width
                            lineHeight: 0.6
                            text: "IP Address"
                        }

                        SmallText {
                            verticalAlignment: Text.AlignVCenter
                            width: 0.35 * parent.width
                            text: root.ipAddress
                        }
                    }
                }
            }
        }

        PropertyTable {
            id: propertyTable
            category: root._categoryName
            monitoring_datas: LK.hosts.getCategoryMonitorIds(root.hostId, root._categoryName)
                                      .map(monitorId => LK.hosts.getMonitoringData(root.hostId, monitorId))
            command_datas: LK.command.getCategoryCommands(root.hostId, root._categoryName)

            Layout.fillHeight: true
            Layout.fillWidth: true

            onButtonClicked: function(buttonId, commandId, params) {
                LK.command.execute(buttonId, root.hostId, commandId, params)
            }
        }
    }
}