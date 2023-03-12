import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

Item {
    id: root
    property string imageSource: ""
    property real imageRelativeWidth: 0.0
    property real imageRelativeHeight: 0.0
    property string color: "transparent"
    property string tooltip: ""
    property bool roundButton: false
    property bool flatButton: true

    width: 0.8 * parent.height
    height: width

    signal clicked()

    Button {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent
        visible: roundButton === false
        onClicked: root.clicked()

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: Theme.tooltip_delay()
        ToolTip.text: root.tooltip

        Image {
            anchors.centerIn: parent
            source: root.imageSource
            width: root.imageRelativeWidth > 0.0 ? root.imageRelativeWidth * root.width :
                                                   getIconRelativeSize(root.imageSource) * root.width
            height: root.imageRelativeHeight > 0.0 ? root.imageRelativeHeight * root.height :
                                                     getIconRelativeSize(root.imageSource) * root.height
            // width: 0.9 * root.width
            // height: 0.9 * root.height

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: root.color
                antialiasing: true
            }
        }
    }

    RoundButton {
        flat: root.flatButton
        anchors.fill: parent
        anchors.centerIn: parent
        visible: roundButton === true
        onClicked: root.clicked()

        ToolTip.visible: root.tooltip !== "" && hovered
        ToolTip.delay: 800
        ToolTip.text: root.tooltip

        Image {
            anchors.centerIn: parent
            source: root.imageSource
            width: root.imageRelativeWidth > 0.0 ? root.imageRelativeWidth * root.width :
                                                   getIconRelativeSize(root.imageSource) * root.width
            height: root.imageRelativeHeight > 0.0 ? root.imageRelativeHeight * root.height :
                                                     getIconRelativeSize(root.imageSource) * root.height

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: root.color
                antialiasing: true
            }
        }
    }

    // TODO: provide this info somehow in qrc or theme file along the icon's path.
    // Icon padding/margins vary a bit so patching a better sizing here.
    function getIconRelativeSize(resourcePath) {
        let icon_name = resourcePath.split("/").pop()
        if (icon_name === "start") {
            return 0.5
        }
        if (icon_name === "stop") {
            return 0.5
        }
        if (icon_name === "delete") {
            return 0.8
        }
        else {
            return 0.9
        }
    }
}