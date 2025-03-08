import QtQuick
import QtQuick.Controls
import Qt5Compat.GraphicalEffects

import Theme

import "../Text"


// TabButton with a close button on tabs.
TabButton {
    id: root
    property int closeButtonSize: 18
    property int iconSize: 22
    property string iconSource: ""
    property bool showCloseButton: false
    property bool active: false
    // Automatically sized to tab title contents.
    width: {
        if (label.text === "") {
            return icon.implicitWidth + (showCloseButton ? closeButton.implicitWidth : 16)
        }
        else {
            return label.implicitWidth + icon.implicitWidth + (showCloseButton ? closeButton.implicitWidth : label.padding)
        }
    }

    signal tabClosed

    // Background and borders.
    background: Item {
        Rectangle {
            anchors.fill: parent
            color: Theme.backgroundColor
        }

        Rectangle {
            width: parent.width
            height: root.active ? 2 : 1
            // Change color if active:
            color: root.active ? palette.highlight : Theme.borderColor
        }

        Rectangle {
            width: 1
            height: parent.height
            color: Theme.borderColor
        }

        Rectangle {
            x: parent.width - 1
            width: 1
            height: parent.height
            color: Theme.borderColor
        }
    }

    contentItem: Item {
        height: root.height
        width: root.width

        // Close with middle-click.
        MouseArea {
            anchors.fill: parent 
            acceptedButtons: Qt.MiddleButton
            onClicked: function(mouse) {
                if (mouse.button === Qt.MiddleButton) {
                    root.tabClosed()
                }
            }
        }

        Row {
            id: contentRow
            anchors.fill: parent

            NormalText {
                id: label
                visible: label.text !== ""
                anchors.verticalCenter: parent.verticalCenter
                text: root.text
                padding: Theme.spacingNormal
            }

            Image {
                id: icon
                visible: root.iconSource !== ""
                source: root.iconSource
                sourceSize.width: 22
                sourceSize.height: 22
                width: root.iconSize
                height: root.iconSize
                anchors.verticalCenter: parent.verticalCenter
            }

            RoundButton {
                id: closeButton
                visible: root.showCloseButton
                flat: true
                focusPolicy: Qt.NoFocus
                // Custom hover effect provided below.
                hoverEnabled: false
                height: root.closeButtonSize
                width: root.closeButtonSize
                anchors.verticalCenter: parent.verticalCenter

                Image {
                    id: defaultImage
                    visible: true
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/close"
                    width: root.closeButtonSize * 0.5
                    height: root.closeButtonSize * 0.8
                }

                ColorOverlay {
                    anchors.fill: defaultImage
                    source: defaultImage
                    // By default this icon is black, so changing it here.
                    color: Theme.iconColor
                    antialiasing: true
                }

                Image {
                    id: hoveredImage
                    visible: false
                    opacity: 0.8
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/tab-close"
                    width: root.closeButtonSize
                    height: root.closeButtonSize
                }

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    preventStealing: true

                    onEntered: {
                        defaultImage.visible = false
                        hoveredImage.visible = true
                    }

                    onExited: {
                        defaultImage.visible = true
                        hoveredImage.visible = false
                    }

                    onClicked: root.tabClosed()
                }
            }
        }
    }
}