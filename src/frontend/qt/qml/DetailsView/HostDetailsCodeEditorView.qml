pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../Button"
import "../js/Utils.js" as Utils


Item {
    id: root
    required property string localFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0
    property string _detectedLanguage: Utils.detectLanguageFromPath(root.localFilePath)
    property string _aceMode: Utils.mapLanguageToAceMode(root._detectedLanguage)
    property var aceEditor: null
    property var aceEditorObject: null
    property var textEditorItem: null
    property bool _useSimpleCodeEditor: false
    property bool disableSaveButton: true

    function _updateUseSimpleCodeEditor() {
        let preferences = LK.config.getPreferences()
        let textEditor = preferences.textEditor || "internal"
        root._useSimpleCodeEditor = textEditor === "internal-simple"
    }

    signal saved(commandId: string, localFilePath: string, content: string)
    signal closed(localFilePath: string)
    signal contentChanged(localFilePath: string, newContent: string)

    onLocalFilePathChanged: {
        root._detectedLanguage = Utils.detectLanguageFromPath(root.localFilePath)
        root._aceMode = Utils.mapLanguageToAceMode(root._detectedLanguage)
    }

    Connections {
        target: LK.hosts

        function onCommandResultReceived(commandResultJson, invocationId) {
            let commandResult = JSON.parse(commandResultJson)

            if (root.pendingInvocation === invocationId) {
                root.pendingInvocation = 0

                if (commandResult.criticality === "Normal") {
                    root.text = commandResult.message
                }

                root.activate()
            }
        }
    }

    Component.onCompleted: {
        root._updateUseSimpleCodeEditor()
        
        // Trigger editor creation if container is ready and simple editor is not selected
        Qt.callLater(function() {
            if (aceEditorContainer && !root._useSimpleCodeEditor) {
                aceEditorContainer.createEditor()
            }
        })
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === ""
    }

    Rectangle {
        id: topBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 34
        color: Theme.backgroundColor
        visible: root.text !== ""

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Theme.spacingNormal
            anchors.rightMargin: Theme.spacingNormal
            anchors.bottomMargin: Theme.spacingTight
            spacing: Theme.spacingTight

            ImageButton {
                size: 0.9 * parent.height
                imageSource: "qrc:/main/images/button/document-save"
                flatButton: true
                tooltip: "Save"
                onClicked: root.save()
                enabled: !root.disableSaveButton
            }

            Item {
                Layout.fillWidth: true
            }
        }
    }

    Item {
        id: aceEditorContainer
        anchors.top: topBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.text !== "" && root.aceEditorObject !== null && !root._useSimpleCodeEditor

        function createEditor() {
            if (root.aceEditorObject !== null || root._useSimpleCodeEditor) {
                return
            }
            
            // Create the AceEditor component dynamically using Qt.createQmlObject
            // This will return null if Lighthouse.AceEditor import is not available
            let aceEditorQml = "import QtQuick\n" +
                               "import QtWebEngine\n" +
                               "import Lighthouse.AceEditor 1.0\n" +
                               "AceEditor {\n" +
                               "anchors.fill: parent\n" +
                               "property var rootItem: null\n" +
                               "onEditorContentChanged: function(newContent) {\n" +
                               "if (rootItem) {\n" +
                               "rootItem.contentChanged(rootItem.localFilePath, newContent)\n" +
                               "rootItem.disableSaveButton = !LK.command.hasFileChanged(rootItem.localFilePath, newContent)\n" +
                               "}\n" +
                               "}\n" +
                               "}"
            let editorObject = Qt.createQmlObject(aceEditorQml, aceEditorContainer, "aceEditor")
            if (editorObject !== null) {
                root.aceEditorObject = editorObject
                editorObject.rootItem = root
                // editor is an alias property in AceEditor component
                root.aceEditor = editorObject["editor"]
                
                // Set content and mode after a brief delay to ensure WebEngineView is ready
                Qt.callLater(function() {
                    if (editorObject) {
                        editorObject.content = root.text
                        editorObject.mode = root._aceMode
                        console.log("Ace editor created successfully, content length:", root.text.length, "mode:", root._aceMode)
                    }
                })
            } else {
                console.log("Ace editor not available: Failed to create Ace editor object")
            }
        }

        Component.onCompleted: {
            if (!root._useSimpleCodeEditor) {
                createEditor()
            }
        }

        Connections {
            target: root

            function on_UseSimpleCodeEditorChanged() {
                if (root._useSimpleCodeEditor && root.aceEditorObject) {
                    root.aceEditorObject.destroy()
                    root.aceEditorObject = null
                    root.aceEditor = null
                } else if (!root._useSimpleCodeEditor && root.aceEditorObject === null) {
                    aceEditorContainer.createEditor()
                }
            }
        }
    }

    Rectangle {
        id: simpleEditorBorder
        anchors.top: topBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.text !== "" && (root.aceEditorObject === null || root._useSimpleCodeEditor)
        color: Theme.backgroundColorLight
        border.width: 1
        border.color: Theme.borderColor

        Loader {
            id: textEditorLoader
            anchors.fill: parent
            anchors.margins: 1
            sourceComponent: (root.aceEditorObject === null || root._useSimpleCodeEditor) ? textEditorComponent : null

            onItemChanged: {
                if (item === null) {
                    root.textEditorItem = null
                }
            }

            Component {
                id: textEditorComponent

                HostDetailsTextEditorView {
                    id: textEditorInstance
                    localFilePath: root.localFilePath
                    text: root.text
                    commandId: root.commandId
                    pendingInvocation: root.pendingInvocation

                    Component.onCompleted: {
                        root.textEditorItem = textEditorInstance
                    }

                    onSaved: function(commandId, localFilePath, content) {
                        root.saved(commandId, localFilePath, content)
                    }
                    onClosed: function(localFilePath) {
                        root.closed(localFilePath)
                    }
                    onContentChanged: function(localFilePath, newContent) {
                        root.contentChanged(localFilePath, newContent)
                        root.disableSaveButton = !LK.command.hasFileChanged(localFilePath, newContent)
                    }
                }
            }
        }
    }

    Text {
        id: errorMessage
        visible: root.text !== "" && root.aceEditorObject === null && !root._useSimpleCodeEditor
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingLoose
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        color: Theme.textColor || "red"
        text: "Integrated code editor could not be loaded. Possibly because of missing Qt web engine installation.\n"+
              "You are using a simple text editor instead. To continue using simple editor without this warning, choose \"internal (simple)\" in settings."
    }

    Shortcut {
        sequence: StandardKey.Save
        onActivated: {
            root.save()
        }
    }

    function save() {
        if (root.commandId === "" || root.localFilePath === "") {
            return
        }

        if (root.aceEditorObject !== null && root.aceEditor && !root._useSimpleCodeEditor) {
            root.aceEditor.getContent(function(content) {
                root.saved(root.commandId, root.localFilePath, content)
            })
        } else if (root.textEditorItem) {
            root.textEditorItem.save()
        }
    }

    function activate() {
        root._updateUseSimpleCodeEditor()
        
        if (root.pendingInvocation === 0) {
            if (root.aceEditorObject !== null && root.aceEditor && !root._useSimpleCodeEditor) {
                root.aceEditor.getContent(function(content) {
                    root.contentChanged(root.localFilePath, content)
                    root.disableSaveButton = !LK.command.hasFileChanged(root.localFilePath, content)
                })
            } else if (root.textEditorItem) {
                root.textEditorItem.activate()
            }
        }
    }

    function deactivate() {
    }

    function refresh() {
    }

    function close() {
        root.closed(root.localFilePath)
    }

    onTextChanged: {
        if (root.aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root.aceEditorObject.content = root.text
        }
    }

    on_AceModeChanged: {
        if (root.aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root.aceEditorObject.mode = root._aceMode
        }
    }
}
