import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0

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
                property bool isFromCache: monitorData.values.slice(-1)[0].is_from_cache
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
                    text: parent.isFromCache ? parent.monitorId + " (CACHED)" : parent.monitorId
                    opacity: 0
                }

                WaveAnimation {
                    anchors.centerIn: statusImage
                    color: parent.color
                    visible: parent.monitorId in root.highlights && !parent.isFromCache
                }

                Rectangle {
                    anchors.fill: statusImage
                    color: Theme.categoryRefreshMask
                    visible: parent.isFromCache

                    MouseArea {
                        anchors.fill: parent
                        preventStealing: true
                    }
                }
            }

        }
    }
}