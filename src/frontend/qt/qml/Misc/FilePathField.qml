/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Dialogs
import QtCore

import Lightkeeper 1.0

import "../Button"
import "../StyleOverride"


RowLayout {
    id: root

    property alias text: textField.text
    property alias placeholderText: textField.placeholderText
    property alias placeholderTextColor: textField.placeholderTextColor

    spacing: Theme.spacingNormal

    TextField {
        id: textField
        readOnly: true
        selectByMouse: false
        Layout.fillWidth: true
    }

    ImageButton {
        imageSource: "qrc:/main/images/button/document-open-folder"
        size: textField.implicitHeight * 0.8
        tooltip: "Choose file"
        enabled: root.enabled
        onClicked: fileDialogLoader.active = true

        Layout.preferredWidth: textField.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    // Using loader because some backends leave background metadata tasks running even after closing dialog
    Loader {
        id: fileDialogLoader
        active: false

        sourceComponent: Component {

            FileDialog {
                title: "Select file"
                currentFolder: StandardPaths.writableLocation(StandardPaths.HomeLocation)

                Component.onCompleted: open()

                onAccepted: {
                    let path = selectedFile.toString()
                    if (path.indexOf("file://") === 0) {
                        path = path.substring(7)
                    }
                    if (path.length > 0) {
                        root.text = path
                    }
                    fileDialogLoader.active = false
                }

                onRejected: fileDialogLoader.active = false
            }
        }
    }
}
