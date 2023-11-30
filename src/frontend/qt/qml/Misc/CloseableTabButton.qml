import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15

import "../Button"


// TabButton with a close button on tabs.
TabButton {
    id: root
    property int closeButtonSize: 18

    signal tabClosed

    contentItem: RowLayout {
        id: contentRow

        Item {
            Layout.alignment: Qt.AlignRight
            Layout.rightMargin: 4
            Layout.topMargin: 2

            height: root.closeButtonSize
            width: root.closeButtonSize

            RoundButton {
                id: closeButton
                // anchors.centerIn: parent
                flat: true
                focusPolicy: Qt.NoFocus
                // Custom hover effect provided below.
                hoverEnabled: false
                height: root.closeButtonSize
                width: root.closeButtonSize

                onClicked: root.tabClosed()


                Image {
                    id: defaultImage
                    visible: true
                    anchors.centerIn: parent
                    source: "qrc:/main/images/button/close"
                    width: root.closeButtonSize * 0.5
                    height: root.closeButtonSize * 0.8

                    ColorOverlay {
                        anchors.fill: parent
                        source: parent
                        // By default this icon is black, so changing it here.
                        color: Theme.iconColor
                        antialiasing: true
                    }
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
                }
            }
        }
    }

}