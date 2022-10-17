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
    property real _subviewSize: 0.0

    // Only one subview can be open at one time, but in case a DetailsDialog is opened using openInNewWindowClicked(),
    // we need to provide the invocation id for state updates since there can be multiple dialogs open.
    property string _subviewInvocationId: ""

    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal openInNewWindowClicked(invocationId: string, text: string, errorText: string, criticality: string)

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    Header {
        id: mainViewHeader
        text: root.hostId
        onMaximizeClicked: root.maximizeClicked()
        onMinimizeClicked: root.minimizeClicked()
        onCloseClicked: root.closeClicked()
    }

    HostDetailsMainView {
        id: detailsMainView
        anchors.top: mainViewHeader.bottom
        anchors.bottom: root.bottom
        anchors.left: root.left
        anchors.right: root.right
        anchors.margins: 10

        commandHandler: root.commandHandler
        hostDataManager: root.hostDataManager
        hostId: root.hostId
    }

    Item {
        id: detailsSubview
        height: (root.height - mainViewHeader.height - 3) * root._subviewSize
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        Header {
            id: subviewHeader

            showOpenInWindowButton: true
            showMaximizeButton: false
            onOpenInWindowClicked: {
                root.openInNewWindowClicked(root._subviewInvocationId, subviewContent.text, subviewContent.errorText, subviewContent.criticality)
                animateHideSubview.start()
            }
            onCloseClicked: animateHideSubview.start()
        }

        HostDetailsSubview {
            id: subviewContent
            anchors.top: subviewHeader.bottom
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right
        }

    }

    NumberAnimation {
        id: animateShowSubview
        target: root
        property: "_subviewSize"
        to: 1.0
        duration: 150
    }

    NumberAnimation {
        id: animateHideSubview
        target: root
        property: "_subviewSize"
        to: 0.0
        duration: 150
    }

    states: [
        State {
            name: "subviewShownVisibility"
            when: root._subviewSize > 0.01

            PropertyChanges {
                target: detailsSubview
                visible: true
            }
        },
        State {
            name: "subviewHiddenVisibility"
            when: root._subviewSize < 0.01

            PropertyChanges {
                target: detailsSubview
                visible: false
            }
        }
    ]

    function refresh() {
        detailsMainView.refresh()
    }

    function openSubview(headerText, invocationId) {
        subviewHeader.text = headerText
        root._subviewInvocationId = invocationId

        animateShowSubview.start()
    }

    function refreshSubview(commandResult) {
        subviewContent.text = commandResult.message
        subviewContent.errorText = commandResult.error
        subviewContent.criticality = commandResult.criticality
    }

}