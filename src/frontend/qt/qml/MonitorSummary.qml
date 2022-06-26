import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

Item {
    id: root
    implicitHeight: parent.height
    implicitWidth: parent.width
    required property var model 
    property var colors: {}
    property var icons: {}

    FontLoader { id: font_label; source: "../fonts/Pixeloid/PixeloidSans-nR3g1.ttf" }

    Row {
        Repeater {
            model: root.model
    
            Item {
                property var monitorData: JSON.parse(modelData)
                property string criticality: JSON.parse(modelData).values[0].criticality.toLowerCase()
                height: root.height
                width: 0.7 * root.height

                states: State {
                    name: "show"
                    when: mouseArea.containsMouse

                    PropertyChanges {
                        target: label
                        opacity: 1
                    }
                }

                Image {
                    id: status_image
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 0.45 * root.height
                    height: 0.45 * root.height
                    antialiasing: true
                    source: getIcon(criticality)

                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                    }
                }

                ColorOverlay {
                    anchors.fill: status_image
                    source: status_image
                    color: getColor(criticality)
                    antialiasing: true
                }

                Text {
                    id: label
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: monitorData.display_options.display_name
                    font.family: font_label.name
                    font.pointSize: 6
                    opacity: 0
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