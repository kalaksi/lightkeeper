import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import Qt.labs.qmlmodels 1.0
import QtGraphicalEffects 1.15

import ".."
import "../Text"
import "../js/TextTransform.js" as TextTransform
import "../js/Parse.js" as Parse
import "../js/ValueUnit.js" as ValueUnit

Item {
    id: root
    required property string hostId
    property bool showSubview: false


    signal closeClicked()
    signal maximizeClicked()
    signal minimizeClicked()


    Connections {
        target: CommandHandler

        function onDetailsSubviewOpened(headerText, invocationId) {
            openTextView(headerText, invocationId)
        }

        function onLogsSubviewOpened(commandId, commandParams, invocationId) {
            openLogView(commandId, commandParams, invocationId)
        }

        // For integrated text editor (not external).
        function onTextEditorSubviewOpened(commandId, invocationId, localFilePath) {
            openTextEditorView(commandId, invocationId, localFilePath)
        }

        // For integrated terminal.
        function onTerminalSubviewOpened(commandId, command) {
            openTerminalView(commandId, command)
        }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.color_background()
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
        onCloseClicked: {
            root.closeSubview()
            root.closeClicked()
        }
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
        height: root.showSubview ? (root.height - mainViewHeader.height - Theme.spacing_loose() / 2) : 0
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right

        Behavior on height {
            NumberAnimation {
                duration: Theme.animation_duration()
                easing.type: Easing.InOutQuad
            }
        }

        Header {
            id: subviewHeader

            // TODO:
            // showOpenInWindowButton: true
            showSaveButton: textEditor.visible
            showCloseButton: true
            onOpenInWindowClicked: {
                root.openInNewWindowClicked(subviewContent.text, subviewContent.errorText, subviewContent.criticality)
                root.closeSubview()
            }
            onCloseClicked: root.closeSubview()
            onSaveClicked: textEditor.save()
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

            HostDetailsTextEditorView {
                id: textEditor
                anchors.fill: parent
                visible: false

                // TODO: discard if not saving on close.
                onSaved: function(commandId, localFilePath, content) {
                    let _invocationId = CommandHandler.saveAndUploadFile(
                        root.hostId,
                        commandId,
                        localFilePath,
                        content
                    )
                    root.closeSubview()
                }
            }

            HostDetailsTerminalView {
                id: terminalView
                anchors.fill: parent
                visible: false
            }
        }
    }

    Shortcut {
        sequence: StandardKey.Cancel
        onActivated: {
            if (root.showSubview) {
                root.closeSubview()
            }
            else {
                root.closeClicked()
            }
        }
    }

    Timer {
        id: hideSubContent
        interval: Theme.animation_duration()
        onTriggered: {
            textView.visible = false
            logView.visible = false
            textEditor.visible = false
            terminalView.close()
        }
    }

    function refresh() {
        detailsMainView.refresh()
    }

    function openTextView(headerText, invocationId) {
        subviewHeader.text = headerText
        textView.open(invocationId)
        root.showSubview = true
    }

    function openLogView(commandId, commandParams, invocationId) {
        subviewHeader.text = commandId
        logView.open(commandId, commandParams, invocationId)
        root.showSubview = true
    }

    function openTextEditorView(commandId, invocationId, localFilePath) {
        subviewHeader.text = commandId
        textEditor.open(commandId, invocationId, localFilePath)
        root.showSubview = true
    }

    function openTerminalView(commandId, command) {
        subviewHeader.text = commandId
        terminalView.open(command)
        root.showSubview = true
    }

    function closeSubview() {
        root.showSubview = false
        hideSubContent.start()
    }

    function openSubview() {
        root.showSubview = true
    }
}