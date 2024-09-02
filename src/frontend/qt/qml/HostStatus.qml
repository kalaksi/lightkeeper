import QtQuick 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import Qt5Compat.GraphicalEffects

import "Text"

Item {
    id: root
    required property string status
    property var colors: {}
    property bool showIcon: true
    anchors.fill: parent

    Component.onCompleted: {
        colors = {
            up: "forestgreen",
            down: "firebrick",
            _: "orange",
        }
    }

    FontLoader {
        id: fontStatus
        source: "qrc:/main/fonts/pressstart2p"
    }

    RowLayout {
        anchors.fill: parent

        Image {
            antialiasing: true
            source: "qrc:/main/images/status/" + root.status
            visible: showIcon

            Layout.leftMargin: showIcon ? 0.4 * parent.height : 0
            Layout.rightMargin: showIcon ? 0.4 * parent.height : 0
            Layout.preferredWidth: showIcon ? 0.7 * parent.height : 0
            Layout.preferredHeight: showIcon ? 0.7 * parent.height : 0
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: getColor(root.status)
                antialiasing: true
                visible: showIcon
            }
        }

        NormalText {
            text: status.toUpperCase()
            font.family: fontStatus.name
            color: Theme.criticalityColor(root.status === "up" ? "normal" : root.status === "down" ? "error" : "_")

            Layout.fillWidth: true
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
        }
    }

    function getColor(status) {
        let result = colors[status]
        if (typeof result === "undefined") {
            return colors["_"]
        }
        return result
    }
}