import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import "Text"

Item {
    id: root
    required property var model 
    required property var highlights

    property var _colors: {}
    Component.onCompleted: function() {
        _colors = {
            critical: "firebrick",
            error: "firebrick",
            warning: "orange",
            normal: "forestgreen",
            info: "forestgreen",
            _: "orange",
        }
    }

    anchors.fill: parent

    Row {
        Repeater {
            model: root.model
    
            Item {
                property var monitorData: JSON.parse(modelData)
                property string criticality: monitorData.values.slice(-1)[0].criticality.toLowerCase()
                property string monitorId: monitorData.display_options.display_text.toLowerCase()
                property string color: getColor(criticality)
                height: root.height
                width: 0.7 * root.height

                states: State {
                    name: "showLabel"
                    when: mouseArea.containsMouse

                    PropertyChanges {
                        target: label
                        opacity: 0.8
                    }
                }

                Image {
                    id: statusImage
                    anchors.centerIn: parent
                    width: 0.45 * root.height
                    height: 0.45 * root.height
                    antialiasing: true
                    source: "qrc:/main/images/criticality/" + parent.criticality

                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                    }
                }

                ColorOverlay {
                    anchors.fill: statusImage
                    source: statusImage
                    color: parent.color
                    antialiasing: true
                }

                PixelatedText {
                    id: label
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: parent.monitorId
                    opacity: 0
                }

                PingAnimation {
                    anchors.centerIn: statusImage
                    color: parent.color
                    visible: parent.monitorId in root.highlights
                }

            }
        }
    }

    function getColor(criticality) {
        let color = _colors[criticality]
        if (typeof color !== "undefined") {
            return color
        }
        return _colors["_"]
    }
}