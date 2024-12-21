import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../Text"
import "../js/TextTransform.js" as TextTransform


GroupBox {
    id: root

    default property alias content: contentItem.data
    property string categoryName: ""
    property alias refreshProgress: groupBoxLabel.refreshProgress
    property bool blocked: refreshProgress < 100

    leftPadding: Theme.spacingTight
    rightPadding: Theme.spacingTight

    background: Rectangle {
        color: Theme.categoryBackgroundColor
    }


    // Custom label provides more flexibility.
    label: GroupBoxLabel {
        id: groupBoxLabel
        anchors.left: root.left
        anchors.right: root.right

        text: TextTransform.capitalize(categoryName)
        icon: Theme.categoryIcon(categoryName)
        color: Theme.categoryColor(categoryName)

        showRefreshButton: true
        onRefreshClicked: refreshClicked()
    }

    signal refreshClicked()


    // Child components get put here.
    Item {
        id: contentItem
        anchors.fill: parent
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.categoryRefreshMask
        visible: root.blocked

        MouseArea {
            anchors.fill: parent
            preventStealing: true
        }
    }
}