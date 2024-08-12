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
    property int selectedRow: -1
    property int buttonSize: 32


    contentItem: ColumnLayout {
        id: content
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        NormalText {
            text: "Certificate monitor allows you to monitor the validity of certificates of your services."
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
                validator: RegularExpressionValidator {
                    regularExpression: /[a-zA-Z\d\-\.\:]+/
                }

                Layout.fillWidth: true
            }

            ImageButton {
                id: addButton
                enabled: addressField.text.length > 0
                imageSource: "qrc:/main/images/button/add"
                size: root.buttonSize
                onClicked: {
                    LK.config.addCertificateMonitor(addressField.text)
                    addressField.text = ""
                    root.refresh()
                }
            }

            ImageButton {
                id: removeButton
                enabled: root.selectedRow >= 0
                imageSource: "qrc:/main/images/button/remove"
                size: root.buttonSize
                onClicked: {
                    let address = table.model[root.selectedRow]
                    LK.config.removeCertificateMonitor(address)
                    root.refresh()
                }
            }

            ImageButton {
                id: refreshButton
                imageSource: "qrc:/main/images/button/refresh"
                size: root.buttonSize
                onClicked: {
                    LK.command.refreshCertificateMonitors()
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