import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import "js/TextTransform.js" as TextTransform
import "js/Parse.js" as Parse
import "js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property var commandHandler
    required property var hostDataManager
    property string hostId: ""

    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    Header {
        id: detailsHeader
        text: root.hostId
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.closeClicked()
    }

    HostDetailsView {
        id: detailsView
        anchors.top: detailsHeader.bottom
        anchors.margins: 5

        commandHandler: root.commandHandler
        hostDataManager: root.hostDataManager
        hostId: root.hostId

    }

    function refresh() {
        detailsView.refresh()
    }

}