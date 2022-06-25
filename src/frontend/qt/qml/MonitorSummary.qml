import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

Item {
    id: root
    implicitHeight: parent.height
    implicitWidth: parent.width
    required property var model 
    property var colors: {}
    property var icons: {}

    Row {
        Repeater {
            model: root.model
    
            Item {
                property string criticality: JSON.parse(modelData).values[0].criticality.toLowerCase()
                height: root.height
                width: 0.75 * root.height

                Image {
                    id: status_image
                    width: 0.5 * root.height
                    height: 0.5 * root.height
                    anchors.verticalCenter: parent.verticalCenter
                    antialiasing: true
                    source: getIcon(criticality)
                }

                ColorOverlay {
                    anchors.fill: status_image
                    source: status_image
                    color: getColor(criticality)
                    antialiasing: true
                }
            }
        }
    }

    Component.onCompleted: function() {
        colors = {
            critical: "#b22222",
            error: "#b22222",
            warning: "orange",
            normal: "green",
            _: "orange",
        }

        icons = {
            critical: "../images/fontawesome/circle-exclamation.svg",
            error: "../images/fontawesome/circle-exclamation.svg",
            warning: "../images/fontawesome/circle-exclamation.svg",
            normal: "../images/fontawesome/circle-check.svg",
            _: "../images/fontawesome/circle-question.svg",
        }
    }

    function getIcon(criticality) {
        let icon = icons[criticality]
        if (typeof icon !== "undefined") {
            return icon 
        }
        return icons["_"]
    }

    function getColor(criticality) {
        let color = colors[criticality]
        if (typeof color !== "undefined") {
            return color
        }
        return colors["_"]
    }

}