import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Qt.labs.qmlmodels
import Qt5Compat.GraphicalEffects

import "../Button"
import "../Text"
import "../Misc"
import "../js/Utils.js" as Utils
import ".."
import "../StyleOverride"

LightkeeperDialog {
    id: root
    title: "Certificate Monitor"
    implicitWidth: 950
    implicitHeight: 650
    standardButtons: Dialog.Close

    property string certMonitorId: LK.hosts.getCertificateMonitorHostId()
    property var dataPoints: {}
    property var certificateMonitors: LK.config.getCertificateMonitors()
    property int tableRowHeight: 60
    property int buttonSize: 32
    property int refreshProgress: 100

    Component.onCompleted: {
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


    contentItem: ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        NormalText {
            text: "Certificate monitor allows you to monitor the validity and expiration of certificates."
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
                enabled: root.refreshProgress === 100
                spinning: root.refreshProgress < 100
                size: root.buttonSize
                flatButton: false
                onClicked: {
                    LK.command.refreshCertificateMonitors()
                    root.refreshProgress = 0
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
                                // TODO: produces warnings about undefined value when adding new entries.
                                text: root.dataPoints[modelData].label
                            }

                            SmallerText {
                                // TODO: produces warnings about undefined value when adding new entries.
                                text: root.dataPoints[modelData].description
                                visible: text !== ""
                                wrapMode: Text.WordWrap
                                width: parent.width
                            }
                        }

                        OptionalText {
                            width: parent.width * 0.43
                            anchors.verticalCenter: parent.verticalCenter
                            placeholder: "No expiration info"
                            text: root.dataPoints[modelData].value
                        }

                        OverlayImage {
                            id: statusImage
                            anchors.verticalCenter: parent.verticalCenter
                            width: 0.45 * parent.height
                            height: 0.45 * parent.height
                            antialiasing: true
                            color: Theme.criticalityColor(root.dataPoints[modelData].criticality)
                            source: "qrc:/main/images/criticality/" + (root.dataPoints[modelData].criticality || "nodata").toLowerCase()
                        }

                        // WaveAnimation {
                        //     anchors.centerIn: statusImage
                        //     color: parent.color
                        //     visible: parent.monitorId in root.highlights && !parent.isFromCache
                        // }
                    }
                }
            }
        }
    }

    function refresh() {
        root.certificateMonitors = LK.config.getCertificateMonitors()
        root.refreshProgress = LK.hosts.getPendingMonitorCount(certMonitorId) > 0 ?  0 : 100

        let monitoringData = JSON.parse(LK.hosts.getMonitoringDataJson(root.certMonitorId, root.certMonitorId))
        let parentDataPoint = monitoringData.values.slice(-1)[0]

        let newDataPoints = {}
        for (const address of root.certificateMonitors) {
            let dataPoint = parentDataPoint.multivalue.find((item) => item.label === address)
            if (dataPoint) {
                newDataPoints[address] = dataPoint
            }
            else {
                newDataPoints[address] = {
                    label: address,
                    value: "",
                    criticality: "nodata",
                    description: ""
                }
            }
        }
        root.dataPoints = newDataPoints
    }
}