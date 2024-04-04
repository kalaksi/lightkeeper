import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import "../StyleOverride"
import "../Text"


/// Provides more flexible button with icon and text.
Item {
    id: root
    property string imageSource: ""
    property real imageRelativeWidth: 0.0
    property real imageRelativeHeight: 0.0
    property string color: "transparent"
    property string tooltip: ""
    property string text: ""
    property bool roundButton: false
    property bool flatButton: false
    property bool hoverEnabled: true
    property bool enabled: true
    property bool checkable: false
    property real size: 0.8 * parent.height

    height: root.size
    width: root.size + (buttonText.text !== "" ? buttonText.implicitWidth + Theme.spacing_normal() * 3 : 0)

    signal clicked()

    Button {
        flat: root.flatButton
        anchors.fill: parent
        visible: roundButton === false
        enabled: root.enabled
        opacity: Theme.opacity(enabled)
        onClicked: root.clicked()
        focusPolicy: Qt.NoFocus
        hoverEnabled: root.hoverEnabled
        checkable: root.checkable

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: Theme.tooltipDelay
        ToolTip.text: root.tooltip

        Row {
            anchors.centerIn: parent
            spacing: Theme.spacingNormal

            Image {
                source: root.imageSource
                width: getIconWidth()
                height: getIconHeight()

                ColorOverlay {
                    anchors.fill: parent
                    source: parent
                    color: root.color
                    antialiasing: true
                }
            }

            NormalText {
                id: buttonText
                visible: root.text !== ""
                text: root.text
            }
        }
    }

    RoundButton {
        // TODO: For some reason, the hover effect is not working on the RoundButton by default.

        flat: root.flatButton
        anchors.fill: parent
        visible: roundButton === true
        enabled: root.enabled
        opacity: root.enabled ? 1.0 : 0.5
        onClicked: root.clicked()
        focusPolicy: Qt.NoFocus
        hoverEnabled: root.hoverEnabled
        checkable: root.checkable

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: Theme.tooltipDelay
        ToolTip.text: root.tooltip

        Row {
            anchors.centerIn: parent
            spacing: Theme.spacingNormal

            Image {
                source: root.imageSource
                width: getIconWidth()
                height: getIconHeight()

                ColorOverlay {
                    anchors.fill: parent
                    source: parent
                    color: root.color
                    antialiasing: true
                }
            }

            NormalText {
                id: roundButtonText
                visible: root.text !== ""
                text: root.text
            }
        }
    }

    // TODO: provide this info somehow in qrc or theme file along the icon's path.
    /// Icon padding/margins vary a bit so patching a better sizing here.
    function getIconRelativeSize(resourcePath) {
        let icon_name = resourcePath.split("/").pop()
        if (icon_name === "start") {
            return 0.5
        }
        if (icon_name === "stop") {
            return 0.5
        }
        else {
            return 0.8
        }
    }

    function getIconWidth() {
        if (root.imageRelativeWidth > 0.0) {
            return root.imageRelativeWidth * root.height
        }
        else {
            return getIconRelativeSize(root.imageSource) * root.height
        }
    }

    function getIconHeight() {
        if (root.imageRelativeHeight > 0.0) {
            return root.imageRelativeHeight * root.height
        }
        else {
            return getIconRelativeSize(root.imageSource) * root.height
        }
    }
}