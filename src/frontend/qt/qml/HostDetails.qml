import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    required property var model

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    GridLayout {
        id: gridLayout
        anchors.fill: parent
        flow: GridLayout.TopToBottom
        rows: 3

        GroupBox {
            title: "Host"
            Layout.fillHeight: true
            Layout.preferredWidth: 0.4 * parent.width
            Layout.rowSpan: 3
            Layout.alignment: Qt.AlignTop
            
            ColumnLayout {
                anchors.top: parent.top
                implicitWidth: parent.width

                PropertyRow {
                    label: "Status"
                    value: root.model.length > 0 ? root.model[0] : ""
                }

                PropertyRow {
                    label: "Name"
                    value: root.model.length > 0 ? root.model[1] : ""
                }

                PropertyRow {
                    label: "FQDN"
                    value: root.model.length > 0 ? root.model[2] : ""
                }

                PropertyRow {
                    label: "IP address"
                    value: root.model.length > 0 ? root.model[3] : ""
                }
            }

        }
    }

}