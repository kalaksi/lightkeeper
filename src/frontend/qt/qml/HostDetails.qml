import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    required property var model
    property var hostData: lightkeeper_data.get_host_data(lightkeeper_data.selected_row)

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    GridLayout {
        id: grid
        anchors.fill: parent
        flow: GridLayout.TopToBottom
        rows: 3

        GroupBox {
            title: "Host"
            Layout.fillHeight: true
            Layout.preferredWidth: 0.3 * parent.width
            Layout.maximumWidth: 600
            Layout.rowSpan: 3
            Layout.alignment: Qt.AlignTop
            
            ColumnLayout {
                anchors.top: parent.top
                implicitWidth: parent.width

                // TODO: get rid of the manual indexing and length checking
                PropertyRow {
                    label: "Status"
                    value: root.hostData.length > 0 ? root.hostData[0] : ""
                }

                PropertyRow {
                    label: "Name"
                    value: root.hostData.length > 0 ? root.hostData[1] : ""
                }

                PropertyRow {
                    label: "FQDN"
                    value: root.hostData.length > 0 ? root.hostData[2] : ""
                }

                PropertyRow {
                    label: "IP address"
                    value: root.hostData.length > 0 ? root.hostData[3] : ""
                }
            }

        }
 
        Repeater {
            model: root.hostData.length > 0 ? root.model.get_monitor_data(root.hostData[1]) : 0
 
            Item {
                property var monitorData: JSON.parse(modelData)
                Layout.rowSpan: 2
                Layout.preferredWidth: 0.2 * grid.width

                PropertyRow {
                    label: monitorData.display_options.display_name
                    value: monitorData.values[0].value + " " + monitorData.display_options.unit
                }
            }
        }
    }

}