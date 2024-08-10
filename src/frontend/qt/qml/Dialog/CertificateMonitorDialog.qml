import QtQuick 2.15
import QtQuick.Layouts 1.11
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0

import "../StyleOverride"
import "../Button"
import "../Text"
import "../Misc"
import "../js/Utils.js" as Utils
import ".."

// This component should be a direct child of main window.
LightkeeperDialog {
    id: root
    title: "Certificate Monitor"
    implicitWidth: 800
    implicitHeight: 650
    standardButtons: Dialog.Close

    property int tableRowHeight: 50


    contentItem: ColumnLayout {
        id: content
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        ToolBar {
            Layout.fillWidth: true

            background: Rectangle {
                color: "transparent"
            }

            // background: BorderRectangle {
            //     backgroundColor: Theme.backgroundColor
            //     borderColor: Theme.borderColor
            //     borderBottom: 1
            // }

            RowLayout {
                anchors.fill: parent

                ToolButton {
                    icon.source: "qrc:/main/images/button/add"
                    text: "Add monitor"
                    display: AbstractButton.IconOnly
                    onClicked: {
                        LK.config.addCertificateMonitor("https://duckduckgo.com")
                        root.refresh()
                    }
                }

                ToolButton {
                    enabled: true
                    opacity: Theme.opacity(enabled)
                    text: "Remove monitor"
                    display: AbstractButton.IconOnly
                    icon.source: "qrc:/main/images/button/remove"
                    onClicked: {
                        LK.config.removeCertificateMonitor("https://duckduckgo.com")
                        root.refresh()
                    }
                }

                // Spacer
                Item {
                    Layout.fillWidth: true
                }

                ToolButton {
                    display: AbstractButton.IconOnly
                    text: "Refresh"
                    icon.source: "qrc:/main/images/button/refresh"
                    onClicked: {
                        
                    }
                }
            }
        }

        BorderRectangle {
            borderColor: Theme.borderColor
            backgroundColor: Theme.backgroundColor
            border: 1

            Layout.fillWidth: true
            Layout.fillHeight: true

            TableView {
                id: table
                anchors.fill: parent
                clip: true
                // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                boundsBehavior: Flickable.StopAtBounds
                onWidthChanged: forceLayout()
                model: LK.config.getCertificateMonitors()

                delegate: TableCell {
                    padding: 20
                    implicitWidth: table.width
                    implicitHeight: root.tableRowHeight

                    Row {
                        spacing: Theme.spacingNormal
                        anchors.fill: parent

                        NormalText {
                            width: Math.max(parent.width * 0.35, implicitWidth)
                            text: modelData
                            anchors.verticalCenter: parent.verticalCenter

                        }
                    }
                }
            }
        }
    }

    function refresh() {
        table.model = LK.config.getCertificateMonitors()
    }
}