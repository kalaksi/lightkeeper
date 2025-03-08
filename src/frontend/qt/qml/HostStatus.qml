import QtQuick
import QtQuick.Layouts

import Theme

import "Text"
import "Misc"

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

        OverlayImage {
            id: image
            antialiasing: true
            source: "qrc:/main/images/status/" + root.status
            color: root.getColor(root.status)
            visible: root.showIcon

            Layout.leftMargin: root.showIcon ? 0.4 * parent.height : 0
            Layout.rightMargin: root.showIcon ? 0.4 * parent.height : 0
            Layout.preferredWidth: root.showIcon ? 0.7 * parent.height : 0
            Layout.preferredHeight: root.showIcon ? 0.7 * parent.height : 0
            Layout.alignment: Qt.AlignLeft | Qt.AlignVCenter
        }

        NormalText {
            text: root.status.toUpperCase()
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