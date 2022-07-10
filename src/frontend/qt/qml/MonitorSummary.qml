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

    FontLoader { id: font_label; source: "qrc:/main/fonts/pixeloid" }

    Row {
        Repeater {
            model: root.model
    
            Item {
                property var monitorData: JSON.parse(modelData)
                property string criticality: monitorData.values.slice(-1)[0].criticality.toLowerCase()
                height: root.height
                width: 0.7 * root.height

                states: State {
                    name: "show_label"
                    when: mouseArea.containsMouse

                    PropertyChanges {
                        target: label
                        opacity: 0.8
                    }
                }

                Image {
                    id: status_image
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 0.45 * root.height
                    height: 0.45 * root.height
                    antialiasing: true
                    source: "qrc:/main/images/criticality/" + criticality

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
                    text: monitorData.display_options.display_text
                    font.family: font_label.name
                    font.pointSize: 6
                    opacity: 0
                }
            }
        }
    }

    Component.onCompleted: function() {
        colors = {
            critical: "firebrick",
            error: "firebrick",
            warning: "orange",
            normal: "forestgreen",
            _: "orange",
        }
    }

    // TODO: unused?
    function getDisplayValue(monitorData) {
        let last = monitorData.slice(-1)[0]
        if (last === null) {
            return "Error"
        }
        else if (last.value === "") {
            if (["error", "critical"].includes(last.criticality.toLowerCase())) {
                return "Error"
            }
            else {
                return ""
            }
        }

        if (last.value !== "-") {
            return last.value + " " + monitorData.display_options.unit
        }

        return last.value
    }

    function getColor(criticality) {
        let color = colors[criticality]
        if (typeof color !== "undefined") {
            return color
        }
        return colors["_"]
    }

}