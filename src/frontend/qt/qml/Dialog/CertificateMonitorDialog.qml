/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

import Lightkeeper 1.0

import "../Button"
import "../Text"
import "../Misc"
import "../StyleOverride"

LightkeeperDialog {
    id: root
    title: "Certificate Monitor"
    implicitWidth: 950
    implicitHeight: 650
    standardButtons: Dialog.Close

    property string certMonitorId: LK.hosts.getCertificateMonitorHostId()
    property var dataPoints: ({})
    property var certificateMonitors: []
    property int tableRowHeight: 60
    property int buttonSize: 32
    property int refreshProgress: 100
    property int nextAutoRefreshIn: 0
    property alias autoRefreshInterval: autoRefreshButton.autoRefreshInterval

    Component.onCompleted: root.syncAutoRefreshInterval()
    onOpened: {
        root.syncAutoRefreshInterval()
        root.refresh()
    }

    Connections {
        target: LK.hosts

        function onMonitoringDataReceived(hostId, category) {
            if (hostId === root.certMonitorId) {
                root.refresh()
            }
        }
    }

    Connections {
        target: LK

        function onTimersChanged(_hostsSecsLeft, certsSecsLeft) {
            root.nextAutoRefreshIn = certsSecsLeft
        }

        // getPreferences() is not a notifiable binding; re-sync after reload / remote reconnect.
        function onReloaded(_error, _resetHosts) {
            root.syncAutoRefreshInterval()
        }
    }


    contentItem: ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        NormalText {
            text: "Certificate monitor allows you to monitor the validity and expiration of TLS certificates."
            bottomPadding: Theme.spacingLoose
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true

            Label {
                text: "Address"
            }

            TextField {
                id: addressField
                placeholderText: "domain:port"
                placeholderTextColor: Theme.textColorDark
                validator: RegularExpressionValidator {
                    regularExpression: /[a-zA-Z\d\-\.\:]+/
                }
                onAccepted: {
                    if (addressField.text.length > 0) {
                        LK.config.addCertificateMonitor(addressField.text)
                        addressField.text = ""
                        LK.reload()
                        root.refresh()
                    }
                }

                Layout.fillWidth: true
            }

            ImageButton {
                enabled: addressField.text.length > 0
                imageSource: "qrc:/main/images/button/add"
                size: root.buttonSize
                onClicked: {
                    LK.config.addCertificateMonitor(addressField.text)
                    addressField.text = ""
                    LK.reload()
                    root.refresh()
                }
            }

            ImageButton {
                enabled: monitorList.currentIndex >= 0
                imageSource: "qrc:/main/images/button/remove"
                size: root.buttonSize
                onClicked: {
                    let address = monitorList.model[monitorList.currentIndex]
                    LK.config.removeCertificateMonitor(address)
                    LK.reload()
                    root.refresh()
                }
            }

            AutoRefreshButton {
                id: autoRefreshButton
                enabled: root.refreshProgress === 100
                spinning: root.refreshProgress < 100
                size: root.buttonSize
                flatButton: false
                nextAutoRefreshIn: root.nextAutoRefreshIn
                onClicked: {
                    LK.command.refreshCertificateMonitors()
                    root.refreshProgress = 0
                }
                onIntervalSelected: function(intervalSec) {
                    let preferences = LK.config.getPreferences()
                    preferences.certMonitorRefreshInterval = intervalSec
                    LK.config.setPreferences(preferences)
                    LK.reload()
                }

                Layout.leftMargin: Theme.spacingLoose
            }
        }

        Rectangle {
            color: Theme.backgroundColor
            border.color: Theme.borderColor
            border.width: 1

            Layout.fillWidth: true
            Layout.fillHeight: true

            ListView {
                id: monitorList
                anchors.fill: parent
                clip: true
                // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                boundsBehavior: Flickable.StopAtBounds
                onWidthChanged: forceLayout()
                currentIndex: -1
                model: root.certificateMonitors

                delegate: ItemDelegate {
                    implicitHeight: root.tableRowHeight
                    width: parent.width
                    highlighted: ListView.isCurrentItem
                    property var dataPoint: root.dataPoints[modelData] || {
                        label: modelData,
                        value: "",
                        criticality: "nodata",
                        description: ""
                    }
                    onClicked: {
                        monitorList.currentIndex = monitorList.currentIndex === index ? -1 : index
                    }

                    Row {
                        anchors.fill: parent
                        anchors.verticalCenter: parent.verticalCenter
                        padding: Theme.spacingNormal

                        Column {
                            width: parent.width * 0.48
                            anchors.verticalCenter: parent.verticalCenter

                            NormalText {
                                text: dataPoint.label
                            }

                            SmallerText {
                                text: dataPoint.description
                                visible: text !== ""
                                wrapMode: Text.WordWrap
                                width: parent.width
                            }
                        }

                        OptionalText {
                            width: parent.width * 0.43
                            anchors.verticalCenter: parent.verticalCenter
                            placeholder: "No expiration info"
                            text: dataPoint.value
                        }

                        OverlayImage {
                            id: statusImage
                            anchors.verticalCenter: parent.verticalCenter
                            width: 0.45 * parent.height
                            height: 0.45 * parent.height
                            antialiasing: true
                            color: Theme.criticalityColor(dataPoint.criticality)
                            source: "qrc:/main/images/criticality/" + dataPoint.criticality.toLowerCase()
                        }
                    }
                }
            }
        }
    }

    function syncAutoRefreshInterval() {
        root.autoRefreshInterval = LK.config.getPreferences().certMonitorRefreshInterval || 0
    }

    function refresh() {
        let monitors = LK.config.getCertificateMonitors()
        root.refreshProgress = LK.hosts.getPendingMonitorCount(root.certMonitorId) > 0 ? 0 : 100

        let monitoringData = JSON.parse(LK.hosts.getMonitoringDataJson(root.certMonitorId, root.certMonitorId))
        let parentDataPoint = monitoringData.values.slice(-1)[0]
        let multivalue = parentDataPoint ? parentDataPoint.multivalue : []

        let newDataPoints = {}
        for (const address of monitors) {
            let dataPoint = multivalue.find((item) => item.label === address)
            newDataPoints[address] = dataPoint || {
                label: address,
                value: "",
                criticality: "nodata",
                description: ""
            }
        }
        root.dataPoints = newDataPoints
        root.certificateMonitors = monitors
    }
}