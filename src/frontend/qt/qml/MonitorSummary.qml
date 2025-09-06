import QtQuick

import Theme

import "Text"
import "Misc"

Item {
    id: root
    required property var model 
    required property var highlights

    anchors.fill: parent


    Row {
        Repeater {
            model: root.model
    
            Item {
                property var monitorData: JSON.parse(modelData)
                property string criticality: monitorData.values.slice(-1)[0].criticality.toLowerCase()
                property string monitorId: monitorData.display_options.display_text.toLowerCase()
                property string color: Theme.criticalityColor(criticality)
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

                OverlayImage {
                    id: statusImage
                    anchors.centerIn: parent
                    width: 0.45 * root.height
                    height: 0.45 * root.height
                    antialiasing: true
                    color: parent.color
                    source: "qrc:/main/images/criticality/" + parent.criticality

                    MouseArea {
                        id: mouseArea
                        anchors.fill: parent
                        hoverEnabled: true
                    }
                }

                PixelatedText {
                    id: label
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: parent.monitorId
                    opacity: 0
                }

                WaveAnimation {
                    anchors.centerIn: statusImage
                    color: parent.color
                    visible: parent.monitorId in root.highlights
                }
            }
        }
    }
}