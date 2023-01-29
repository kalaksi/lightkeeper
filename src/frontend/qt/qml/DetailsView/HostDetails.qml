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
    property string hostId: ""
    property real _subviewSize: 0.0
    // Only one subview can be open at one time, but in case a DetailsDialog is opened using openInNewWindowClicked(),
    // we need to provide the invocation id for state updates since there can be multiple dialogs open.
    property string _subviewInvocationId: ""

    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()
    signal openInNewWindowClicked(invocationId: string, text: string, errorText: string, criticality: string)

    Component.onCompleted: {
        CommandHandler.details_subview_opened.connect((headerText, invocationId) => {
            openTextView(headerText, invocationId)
        })

        CommandHandler.logs_subview_opened.connect((headerText, invocationId) => {
            openLogView(headerText, invocationId)
        })
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
        onRefreshClicked: CommandHandler.refresh_monitors(root.hostId)
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
        height: (root.height - mainViewHeader.height - 3) * root._subviewSize
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        Header {
            id: subviewHeader

            showOpenInWindowButton: true
            showMinimizeButton: true 
            showMaximizeButton: true
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
                anchors.fill: parent
                visible: false
                selections: []
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

    function openLogView(headerText, invocationId) {
        subviewHeader.text = headerText
        root._subviewInvocationId = invocationId

        let monitorDataJSON = HostDataManager.get_monitor_data(root.hostId, "systemd-service")

        if (monitorDataJSON.length > 2) {
            let monitorData = JSON.parse(monitorDataJSON)
            let lastDataPoint = monitorData.values.slice(-1)[0]
            logView.selections = ["ALL", "DMESG"].concat(
                lastDataPoint.multivalue.map((item) => item.label)
            )

            textView.visible = false
            logView.visible = true
            textEditor.visible = false
            animateShowSubview.start()
        }
    }

    function openTextEditorView(headerText, invocationId) {
        subviewHeader.text = headerText
        root._subviewInvocationId = invocationId

        textView.visible = false
        logView.visible = false
        textEditor.visible = true
    }

    function refreshSubview(commandResult) {
        if (textView.visible) {
            textView.text = commandResult.message
            textView.errorText = commandResult.error
            textView.criticality = commandResult.criticality
        }
        else {
            logView.text = commandResult.message
            logView.errorText = commandResult.error
            logView.criticality = commandResult.criticality
        }
    }

}