import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

Item {
    required property string status
    property var colors: {}
    property bool showIcon: true

    FontLoader { id: font_status; source: "qrc:/main/fonts/pressstart2p" }

    Image {
        id: status_image
        x: 0.4 * parent.height
        width: showIcon ? 0.7 * parent.height : 0
        height: showIcon ? 0.7 * parent.height : 0
        anchors.verticalCenter: parent.verticalCenter
        antialiasing: true
        source: "qrc:/main/images/status/" + parent.status
        visible: showIcon
    }

    ColorOverlay {
        anchors.fill: status_image
        source: status_image
        color: getColor()
        antialiasing: true
        visible: showIcon
    }

    Text {
        anchors.left: status_image.right
        anchors.leftMargin: 0.4 * parent.height
        anchors.rightMargin: 0.4 * parent.height
        anchors.verticalCenter: parent.verticalCenter

        text: status.toUpperCase()
        font.family: font_status.name
        color: getColor()
    }

    Component.onCompleted: function() {
        colors = {
            up: "forestgreen",
            down: "firebrick",
            _: "orange",
        }
    }

    function getColor() {
        let color = colors[status]
        if (typeof color !== "undefined") {
            return color
        }
        return colors["_"]
    }
}