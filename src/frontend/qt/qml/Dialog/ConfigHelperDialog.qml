/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../Text"


/// ConfigHelper notifies user about any new modules that are available for the configuration group,
/// and makes it easy to add those to configuration.
LightkeeperDialog {
    id: root
    required property string groupName
    property int tableRowHeight: 50
    property var missingModules: LK.config.compareToDefault(groupName).map((item) => item.split(",", 2))
    visible: missingModules.length > 0
    bottomMargin: 0.20 * parent.height
    width: parent.width * 0.6
    height: parent.height * 0.7
    title: "Configuration helper"

    standardButtons: Dialog.Yes | Dialog.No

    signal configurationChanged()

    onAccepted: {
        for (let row of table.model) {
            // TODO: ModuleSpecification or something as QML object so we can have proper objects here (and in other places too)
            let parts = row[0].split(": ")
            let moduleType = parts[0]
            let moduleName = parts[1]

            if (moduleType === "Monitor") {
                LK.config.addGroupMonitor(root.groupName, moduleName)
            }
            else if (moduleType === "Command") {
                LK.config.addGroupCommand(root.groupName, moduleName)
            }
            else if (moduleType === "Connector") {
                LK.config.addGroupConnector(root.groupName, moduleName)
            }

        }
        LK.config.writeGroupConfiguration()
        root.configurationChanged()
    }

    onRejected: {
        // TODO: ModuleSpecification or something as QML object so we can have proper objects here (and in other places too)
        let commandModules = table.model.filter((row) => row[0].startsWith("Command: ")).map((row) => row[0].split(": ")[1])
        let monitorModules = table.model.filter((row) => row[0].startsWith("Monitor: ")).map((row) => row[0].split(": ")[1])
        let connectorModules = table.model.filter((row) => row[0].startsWith("Connector: ")).map((row) => row[0].split(": ")[1])
        LK.config.ignoreFromConfigHelper(root.groupName, commandModules, monitorModules, connectorModules)
        LK.config.writeGroupConfiguration()
    }

    contentItem: ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        BigText {
            text: "New features available!"
        }

        NormalText {
            text: `It seems that you are missing these modules from the '${root.groupName}' configuration group:`
        }

        Rectangle {
            color: Theme.backgroundColor
            border.color: Theme.borderColor
            border.width: 1

            Layout.fillWidth: true
            Layout.fillHeight: true

            TableView {
                id: table
                anchors.fill: parent
                clip: true
                // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                boundsBehavior: Flickable.StopAtBounds
                onWidthChanged: forceLayout()
                model: root.missingModules

                delegate: TableCell {
                    padding: 20
                    implicitWidth: table.width
                    implicitHeight: root.tableRowHeight

                    Row {
                        spacing: Theme.spacingNormal
                        anchors.fill: parent

                        NormalText {
                            width: Math.max(parent.width * 0.35, implicitWidth)
                            text: modelData[0]
                            anchors.verticalCenter: parent.verticalCenter

                        }

                        NormalText {
                            text: modelData[1]
                            width: Math.min(parent.width * 0.65, implicitWidth)
                            anchors.verticalCenter: parent.verticalCenter
                            wrapMode: Text.WordWrap
                        }
                    }
                }
            }
        }

        NormalText {
            text: `
To add these modules to the configuration group, select 'yes' (recommended).
To use a more customized configuration, select 'no', and add preferred modules manually.`
        }
    }
}