pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import "../Button"
import "../Misc"

Item {
    id: root
    property string color: "#444444"
    property bool showRefreshButton: false
    property bool showMinimizeButton: false
    property bool showMaximizeButton: false
    property bool showOpenInWindowButton: false
    property bool showSaveButton: false
    property bool disableSaveButton: false
    property bool showCharts: false
    property var tabs: []
    property alias tabIndex: tabBar.currentIndex
    property bool _maximized: false
    property int _oldTabIndex: -1

    implicitWidth: parent.width
    implicitHeight: 34

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
            contentHeight: parent.height * 0.85

            onCurrentIndexChanged: {
                root.tabChanged(root._oldTabIndex, currentIndex)
                root._oldTabIndex = currentIndex
            }

            Layout.leftMargin: Theme.spacingTight
            Layout.alignment: Qt.AlignBottom
            Layout.preferredWidth: parent.width * 0.7

            Repeater {
                model: root.tabs

                LKTabButton {
                    required property string modelData
                    required property int index

                    property bool _hasIcon: modelData.startsWith("qrc:")

                    // Chart and host details tabs can't be closed.
                    showCloseButton: root.showCharts ? index > 1 : index > 0
                    // Handle strings starting with "qrc:" as icons.
                    text: _hasIcon ? "" : modelData
                    iconSource: _hasIcon ? modelData : ""
                    onTabClosed: root.tabClosed(index)
                    active: index === tabBar.currentIndex
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        RefreshButton {
            size: 0.9 * parent.height
            onClicked: root.refreshClicked()
            visible: root.showRefreshButton
        }

        ImageButton {
            size: 0.9 * parent.height
            imageSource: "qrc:/main/images/button/window-new"
            flatButton: true
            tooltip: "Open in new window"
            onClicked: root.openInWindowClicked()
            visible: root.showOpenInWindowButton

            Layout.rightMargin: Theme.spacingNormal
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
            Layout.preferredWidth: Theme.spacingLoose
            Layout.preferredHeight: parent.height
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
            color: Theme.iconColor.toString()
            imageRelativeWidth: 0.5
            imageRelativeHeight: 0.8
            flatButton: true
            tooltip: "Close"
            onClicked: root.closeClicked()
        }
    }

    function selectTab(index) {
        tabBar.setCurrentIndex(index)
    }
}