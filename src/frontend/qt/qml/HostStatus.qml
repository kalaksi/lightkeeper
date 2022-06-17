import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

Item {
    required property string status

    height: 0.8 * parent.height
    width: 0.8 * parent.height

    anchors {
        verticalCenter: parent.verticalCenter
    }

    Image {
        id: status_image
        width: parent.width
        height: parent.height
        antialiasing: true
        source: getIcon()
    }

    ColorOverlay {
        anchors.fill: status_image
        source: status_image
        color: getColor()
    }

    function getIcon() {
        if (status.toLowerCase() === "up") {
            return "../images/fontawesome/circle-arrow-up.svg"
        }
        else if (status.toLowerCase() === "down") {
            return "../images/fontawesome/circle-arrow-down.svg"
        }
        else {
            return "../images/fontawesome/circle-question.svg"
        }
    }

    function getColor() {
        if (status.toLowerCase() === "up") {
            return "green"
        }
        else if (status.toLowerCase() === "down") {
            return "red"
        }
        else {
            return "orange"
        }
    }
}