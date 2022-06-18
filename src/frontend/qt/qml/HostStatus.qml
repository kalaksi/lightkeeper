import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

Item {
    anchors.fill: parent
    x: 0.2 * parent.height

    required property string status
    property var colors: {}
    property var icons: {}

    FontLoader { id: font_status; source: "../fonts/PressStart2P/PressStart2P-vaV7.ttf" }

    Image {
        id: status_image
        x: 0.4 * parent.height
        width: parent.height
        height: parent.height
        antialiasing: true
        source: getIcon()
    }

    ColorOverlay {
        anchors.fill: status_image
        source: status_image
        color: getColor()
        antialiasing: true
    }

    Text {
        anchors.left: status_image.right
        anchors.leftMargin: 0.3 * width
        anchors.rightMargin: 0.3 * width
        anchors.verticalCenter: parent.verticalCenter

        text: status
        font.family: font_status.name
        color: getColor()
    }

    Component.onCompleted: function() {
        colors = {
            up: "green",
            down: "#b22222",
            _: "orange",
        }

        icons = {
            up: "../images/fontawesome/circle-arrow-up.svg",
            down: "../images/fontawesome/circle-arrow-down.svg",
            _: "../images/fontawesome/circle-question.svg",
        }
    }

    function getIcon() {
        let icon = icons[status.toLowerCase()]
        if (typeof icon !== "undefined") {
            return icon 
        }
        return icons["_"]
    }

    function getColor() {
        let color = colors[status.toLowerCase()]
        if (typeof color !== "undefined") {
            return color
        }
        return colors["_"]
    }
}