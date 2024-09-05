import QtQuick
import QtQuick.Controls
import Qt.labs.qmlmodels
import QtQuick.Layouts
import Qt5Compat.GraphicalEffects

import "../Button"
import "../Text"


// TabButton with a close button on tabs.
TabButton {
    id: root
    property int closeButtonSize: 18
    property bool showCloseButton: false
    property bool active: false
    height: 28

    signal tabClosed

    background: Item {
        Rectangle {
            anchors.fill: parent
            color: Theme.backgroundColor
        }

        Rectangle {
            width: parent.width
            height: root.active ? 2 : 1
            // color: Theme.borderColor
            // color: palette.highlight
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

    contentItem: Row {
        id: contentRow
        spacing: 0
        height: root.height

        NormalText {
            id: label
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            padding: Theme.spacingNormal
        }

        Item {
            visible: root.showCloseButton
            height: root.closeButtonSize
            width: root.closeButtonSize * 2
            anchors.verticalCenter: parent.verticalCenter

            RoundButton {
                id: closeButton
                // anchors.centerIn: parent
                flat: true
                focusPolicy: Qt.NoFocus
                // Custom hover effect provided below.
                hoverEnabled: false
                height: root.closeButtonSize
                width: root.closeButtonSize

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