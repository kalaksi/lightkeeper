import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtGraphicalEffects 1.15


Item {
    id: root
    required property string text
    required property string criticality

    property var colors: {}
    property var alertLevelMap: {}

    implicitHeight: textContent.height
    implicitWidth: textContent.width

    RowLayout {
        anchors.fill: parent

        Image {
            id: error_image
            antialiasing: true
            source: "qrc:/main/images/alert/" + criticalityToAlertLevel(root.criticality)

            Layout.leftMargin: 0.4 * parent.height
            Layout.rightMargin: 0.4 * parent.height
            Layout.preferredWidth: 0.7 * parent.height
            Layout.preferredHeight: 0.7 * parent.height
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter

            ColorOverlay {
                anchors.fill: error_image 
                source: error_image 
                color: getColor(root.criticality)
                antialiasing: true
            }
        }

        NormalText {
            id: textContent
            text: root.text
            wrapMode: Text.WordWrap

            Layout.fillWidth: true
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
        }
    }

    Component.onCompleted: function() {
        alertLevelMap = {
            warning: "warning",
            error: "error",
            critical: "error",
            _: "warning",
        }
        colors = {
            warning: "orange",
            error: "firebrick",
            _: "",
        }
    }

    function criticalityToAlertLevel(criticality) {
        let level = alertLevelMap[criticality]
        return typeof level === "undefined" ?  alertLevelMap["_"] : level
    }

    function getColor(criticality) {
        let level = criticalityToAlertLevel(criticality)
        return colors[level]
    }
}