import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

import "../Button"
import "../Text"


Item {
    id: root
    anchors.fill: parent

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
                onClicked: monitorList.currentIndex = monitorList.currentIndex === index ? -1 : index

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