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


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()


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
        visible: root._subviewSize > 0.01
        height: (root.height - mainViewHeader.height - Theme.spacing_loose() / 2) * root._subviewSize
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        Header {
            id: subviewHeader

            // TODO:
            // showOpenInWindowButton: true
            showCloseButton: true
            onOpenInWindowClicked: {
                root.openInNewWindowClicked(subviewContent.text, subviewContent.errorText, subviewContent.criticality)
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

            HostDetailsTextView {
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

    Shortcut {
        sequence: StandardKey.Cancel
        onActivated: {
            if (root._subviewSize > 0.01) {
                root.closeSubview()
            }
            else {
                root.closeClicked()
            }
        }
    }

    function refresh() {
        detailsMainView.refresh()
    }

    function openTextView(headerText, invocationId) {
        subviewHeader.text = headerText

        logView.close()
        textView.open(invocationId)
        textEditor.close()
        animateShowSubview.start()
    }

    function openLogView(commandId, invocationId) {
        subviewHeader.text = commandId

        textView.close()
        textEditor.close()
        logView.open(commandId, invocationId)
        animateShowSubview.start()
    }

    function openTextEditorView(headerText, invocationId) {
        subviewHeader.text = headerText

        textView.close()
        logView.close()
        textEditor.open(invocationId)
        animateShowSubview.start()
    }

    function closeSubview() {
        if (root._subviewSize > 0.01) {
            animateHideSubview.start()
        }

        textView.close()
        textEditor.close()
        logView.close()
    }
}