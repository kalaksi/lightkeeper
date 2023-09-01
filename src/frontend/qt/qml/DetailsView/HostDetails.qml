import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15
import QtQuick.Controls.Material 2.15

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse
import "../js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property string hostId
    property real _subviewSize: 0.0
    // Only one subview can be open at one time, but in case a DetailsDialog is opened using openInNewWindowClicked(),
    // we need to provide the invocation id for state updates since there can be multiple dialogs open.
    property string _subviewInvocationId: ""


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal openInNewWindowClicked(invocationId: string, text: string, errorText: string, criticality: string)


    Connections {
        target: CommandHandler

        function onDetails_subview_opened(headerText, invocationId) {
            openTextView(headerText, invocationId)
        }

        function onLogs_subview_opened(commandId, invocationId) {
            openLogView(commandId, invocationId)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Material.background
    }

    Header {
        id: mainViewHeader
        text: root.hostId
        showRefreshButton: true
        showMinimizeButton: true
        showMaximizeButton: true
        showCloseButton: true
        onRefreshClicked: CommandHandler.force_initialize_host(hostId)
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

        hostId: root.hostId
    }

    Item {
        id: detailsSubview
        height: (root.height - mainViewHeader.height - Theme.spacing_loose() / 2) * root._subviewSize
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        Header {
            id: subviewHeader

            showOpenInWindowButton: true
            showCloseButton: true
            onOpenInWindowClicked: {
                root.openInNewWindowClicked(root._subviewInvocationId, subviewContent.text, subviewContent.errorText, subviewContent.criticality)
                animateHideSubview.start()
            }
            onCloseClicked: animateHideSubview.start()
        }

        Item {
            id: subviewContent
            anchors.top: subviewHeader.bottom
            anchors.bottom: parent.bottom
            anchors.left: parent.left
            anchors.right: parent.right

            HostDetailsTextView {
                id: textView
                anchors.fill: parent
                visible: false
            }

            // TODO: disable button until service unit list is received.
            HostDetailsLogView {
                id: logView
                hostId: root.hostId
                anchors.fill: parent
                visible: false
            }

            HostDetailsTextView{
                id: textEditor
                anchors.fill: parent
                visible: false
            }
        }
    }

    // TODO: custom component (see main.qml too)
    NumberAnimation {
        id: animateShowSubview
        target: root
        property: "_subviewSize"
        to: 1.0
        easing.type: Easing.OutQuad
        duration: 175
    }

    NumberAnimation {
        id: animateHideSubview
        target: root
        property: "_subviewSize"
        to: 0.0
        easing.type: Easing.OutQuad
        duration: 175
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

    function openTextView(headerText, invocationId) {
        subviewHeader.text = headerText
        root._subviewInvocationId = invocationId
        logView.visible = false
        textView.visible = true
        textEditor.visible = false

        animateShowSubview.start()
    }

    function openLogView(commandId, invocationId) {
        subviewHeader.text = commandId
        root._subviewInvocationId = invocationId

        logView.commandId = commandId

        textView.visible = false
        logView.visible = true
        textEditor.visible = false
        animateShowSubview.start()
    }

    function openTextEditorView(headerText, invocationId) {
        subviewHeader.text = headerText
        root._subviewInvocationId = invocationId

        textView.visible = false
        logView.visible = false
        textEditor.visible = true
        animateShowSubview.start()
    }

    function refreshSubview(commandResult) {
        if (textView.visible) {
            // If message seems to contain JSON...
            if (commandResult.message.startsWith("{")) {
                textView.jsonText = commandResult.message
            }
            else {
                textView.text = commandResult.message
            }
            textView.errorText = commandResult.error
            textView.criticality = commandResult.criticality
        }
        else if (logView.visible) {
            logView.text = commandResult.message
            logView.errorText = commandResult.error
            logView.criticality = commandResult.criticality
        }
    }

}