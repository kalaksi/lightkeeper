import QtQuick 2.15


Item {
    id: root
    required property string text
    required property string criticality
    property real imageScale: 2.0

    property var _icons: {}
    Component.onCompleted: _icons = {
        nodata: "qrc:/main/images/alert/warning",
        normal: "qrc:/main/images/alert/information",
        warning: "qrc:/main/images/alert/warning",
        error: "qrc:/main/images/alert/error",
        critical: "qrc:/main/images/alert/error",
    }

    implicitWidth: textContent.contentWidth + image.width
    implicitHeight: Math.max(textContent.implicitHeight, image.height)

    Row {
        id: row
        padding: 20
        spacing: 10
        anchors.verticalCenter: parent.verticalCenter

        Image {
            id: image
            antialiasing: true
            source: getIcon(root.criticality.toLowerCase())
            width: 22 * root.imageScale
            height: 22 * root.imageScale
            anchors.verticalCenter: parent.verticalCenter
        }

        NormalText {
            id: textContent
            anchors.verticalCenter: parent.verticalCenter
            text: root.text
            wrapMode: Text.Wrap
            width: root.width - image.width  - row.spacing - row.padding * 2
        }
    }

    function getIcon(criticality) {
        if (criticality in root._icons) {
            return root._icons[criticality]
        }
        return root._icons["nodata"]
    }
}
