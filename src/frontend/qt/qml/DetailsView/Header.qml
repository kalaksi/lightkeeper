import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15

import "../Button"
import "../Text"
import "../Misc"

Item {
    id: root
    property string color: "#444444"
    property bool showRefreshButton: false
    property bool showMinimizeButton: false
    property bool showMaximizeButton: false
    property bool showCloseButton: false
    property bool showOpenInWindowButton: false
    property bool showSaveButton: false
    property bool disableSaveButton: false
    property int pendingInvocations: 0
    property var tabs: []
    property alias tabIndex: tabBar.currentIndex
    property bool _maximized: false
    property int _oldTabIndex: -1

    implicitWidth: parent.width
    implicitHeight: 32

    signal refreshClicked()
    signal openInWindowClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal closeClicked()
    signal saveClicked()
    signal tabClosed(int index)
    signal tabChanged(int oldIndex, int newIndex)



    Rectangle {
        color: root.color
        anchors.fill: parent
    }

    RowLayout {
        anchors.fill: parent
        anchors.rightMargin: Theme.spacingTight
        spacing: Theme.spacingTight

        TabBar {
            id: tabBar
            width: Math.floor(parent.width * 0.7)
            height: parent.height

            onCurrentIndexChanged: {
                root.tabChanged(root._oldTabIndex, currentIndex)
                root._oldTabIndex = currentIndex
            }

            Layout.leftMargin: Theme.spacingTight
            Layout.alignment: Qt.AlignBottom

            Repeater {
                model: root.tabs

                CloseableTabButton {
                    // First tab can't be closed.
                    showCloseButton: index > 0
                    text: modelData
                    onTabClosed: root.tabClosed(index)
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        SmallText {
            id: jobsText
            rightPadding: Theme.spacingLoose * 2
            text: root.pendingInvocations + " jobs left"
            color: root.pendingInvocations > 0 ? Theme.textColor : Theme.disabledTextColor
        }

        RefreshButton {
            size: 0.9 * parent.height
            onClicked: root.refreshClicked()
            visible: root.showRefreshButton
        }

        ImageButton {
            size: 0.9 * parent.height
            anchors.rightMargin: Theme.spacingNormal
            imageSource: "qrc:/main/images/button/window-new"
            flatButton: true
            tooltip: "Open in new window"
            onClicked: root.openInWindowClicked()
            visible: root.showOpenInWindowButton
        }

        ImageButton {
            size: 0.9 * parent.height
            imageSource: "qrc:/main/images/button/document-save"
            flatButton: true
            tooltip: "Save"
            onClicked: root.saveClicked()
            enabled: !root.disableSaveButton
            visible: root.showSaveButton
        }

        // Spacer
        Item {
            width: Theme.spacingLoose
            height: parent.height
        }

        ImageButton {
            size: 0.9 * parent.height
            imageSource: "qrc:/main/images/button/maximize"
            flatButton: true
            tooltip: "Maximize"
            onClicked: {
                root.maximizeClicked()
                root._maximized = true
            }
            visible: root.showMaximizeButton && !root._maximized
        }

        ImageButton {
            size: 0.9 * parent.height
            imageSource: "qrc:/main/images/button/minimize"
            flatButton: true
            tooltip: "Minimize"
            onClicked: {
                root.minimizeClicked()
                root._maximized = false
            }
            visible: root.showMinimizeButton && root._maximized
        }

        ImageButton {
            size: 0.9 * parent.height
            imageSource: "qrc:/main/images/button/close"
            // By default this icon is black, so changing it here.
            color: Theme.iconColor
            imageRelativeWidth: 0.5
            imageRelativeHeight: 0.8
            flatButton: true
            tooltip: "Close"
            onClicked: root.closeClicked()
            visible: root.showCloseButton
        }
    }

    function selectTab(index) {
        tabBar.setCurrentIndex(index)
    }
}