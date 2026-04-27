/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0


LightkeeperDialog {
    id: root
    title: "Lightkeeper Core"
    implicitWidth: 520
    implicitHeight: 420
    standardButtons: Dialog.Ok | Dialog.Cancel

    property var savedAddresses: []

    onOpened: {
        root.refreshSaved()
    }

    onAccepted: {
        let trimmed = addressField.text.trim()
        if (trimmed.length > 0) {
            LK.config.addSavedCoreAddress(trimmed)
            root.refreshSaved()
        }
    }

    function refreshSaved() {
        root.savedAddresses = LK.config.getSavedCoreAddresses()
    }

    contentItem: ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingNormal

            TextField {
                id: addressField
                Layout.fillWidth: true
                placeholderText: "address:port"
                placeholderTextColor: Theme.textColorDark
            }

            Button {
                text: "Test"
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/network-disconnect"
                icon.height: 22
                icon.width: 22
                onClicked: {
                }
            }

            Button {
                text: "Connect"
                display: AbstractButton.TextBesideIcon
                icon.source: "qrc:/main/images/button/network-connect"
                icon.height: 22
                icon.width: 22
                onClicked: {
                }
            }
        }


        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Theme.backgroundColor
            border.color: Theme.borderColor
            border.width: 1

            ListView {
                id: savedList
                anchors.fill: parent
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                currentIndex: -1
                model: root.savedAddresses

                delegate: ItemDelegate {
                    implicitHeight: 44
                    width: savedList.width
                    text: modelData
                    highlighted: ListView.isCurrentItem

                    onClicked: {
                        addressField.text = modelData
                    }
                }
            }
        }
    }
}
