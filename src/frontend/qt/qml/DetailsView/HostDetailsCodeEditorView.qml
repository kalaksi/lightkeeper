pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Theme

import ".."
import "../Button"
import "../Misc"
import "../js/Utils.js" as Utils


Item {
    id: root
    required property string localFilePath
    property var text: ""
    property string commandId: ""
    property int pendingInvocation: 0
    property var textEditorItem: null
    property bool disableSaveButton: true
    property string editMode: "regular"
    property int fontSize: 12
    property var _aceEditorObject: null
    property bool _useSimpleCodeEditor: false
    property string _detectedLanguage: Utils.detectLanguageFromPath(root.localFilePath)
    property string _aceMode: Utils.mapLanguageToAceMode(root._detectedLanguage)

    signal saved(commandId: string, localFilePath: string, content: string)
    signal closed(localFilePath: string)
    signal contentChanged(localFilePath: string, newContent: string)

    onLocalFilePathChanged: {
        root._detectedLanguage = Utils.detectLanguageFromPath(root.localFilePath)
        root._aceMode = Utils.mapLanguageToAceMode(root._detectedLanguage)
    }

    onTextChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.content = root.text
        }
    }

    on_AceModeChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.mode = root._aceMode
        }
    }

    onEditModeChanged: {
        if (!root._useSimpleCodeEditor) {
            root._setEditorKeybindings()
        }
        root._saveEditorPreferences()
    }

    onFontSizeChanged: {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.setEditorOption("fontSize", root.fontSize)
        }
        root._saveEditorPreferences()
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
        root._loadEditorPreferences()
        
        if (!root._useSimpleCodeEditor) {
            aceEditorContainer.createEditor()
        }
    }

    Rectangle {
        color: Theme.backgroundColorLight
        anchors.fill: parent
    }

    WorkingSprite {
        visible: root.text === ""
    }

    ToolBar {
        id: topBar
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: 0
        anchors.bottomMargin: Theme.spacingLoose
        height: 36
        visible: root.text !== ""

        background: BorderRectangle {
            backgroundColor: Theme.backgroundColor
            borderColor: Theme.borderColor
            borderBottom: 1
        }

        RowLayout {
            width: parent.width
            height: parent.height
            anchors.top: parent.top
            spacing: Theme.spacingNormal

            ToolButton {
                icon.source: "qrc:/main/images/button/document-save"
                text: "Save"
                display: AbstractButton.IconOnly
                onClicked: root.save()
                enabled: !root.disableSaveButton
                icon.height: 24
                icon.width: 24
                padding: 4
            }

            ToolSeparator {
            }

            Text {
                text: "Font"
                color: Theme.textColor
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            ComboBox {
                id: fontSizeComboBox
                model: [10, 11, 12, 13, 14, 15, 16, 18, 20, 22, 24]
                currentIndex: {
                    let index = model.indexOf(root.fontSize)
                    return index >= 0 ? index : 2
                }
                onCurrentIndexChanged: {
                    root.fontSize = model[currentIndex]
                }
                Layout.preferredWidth: 80
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            Item {
                Layout.fillWidth: true
            }

            Text {
                text: "Edit mode"
                color: Theme.textColor
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }

            ComboBox {
                id: editModeComboBox
                model: ["regular", "vim", "emacs"]
                currentIndex: {
                    switch (root.editMode) {
                        case "regular": return 0
                        case "vim": return 1
                        case "emacs": return 2
                        default: return 0
                    }
                }
                onCurrentIndexChanged: {
                    root.editMode = model[currentIndex]
                }
                Layout.preferredWidth: 120
                Layout.alignment: Qt.AlignVCenter
                visible: root._aceEditorObject !== null && !root._useSimpleCodeEditor
            }
        }
    }

    Item {
        id: aceEditorContainer
        anchors.top: topBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.text !== "" && root._aceEditorObject !== null && !root._useSimpleCodeEditor

        function createEditor() {
            if (root._aceEditorObject !== null || root._useSimpleCodeEditor) {
                return
            }
            
            // Create the AceEditor component dynamically.
            // This will return null if Lighthouse.AceEditor import is not available.
            let aceEditorQml = `
                import QtQuick;
                import QtWebEngine;
                import Lighthouse.AceEditor 1.0;
                AceEditor {
                    anchors.fill: parent;
                    property var rootItem: null;
                    defaultBackgroundColor: "#1d1f21"

                    onEditorContentChanged: function(newContent) {
                        if (rootItem) {
                            rootItem.contentChanged(rootItem.localFilePath, newContent);
                            rootItem.disableSaveButton = !LK.command.hasFileChanged(rootItem.localFilePath, newContent);
                        }
                    }

                    // TODO
                    // onSaved: {
                    //     if (rootItem) {
                    //         rootItem.save()
                    //     }
                    // }

                    // onClosed: {
                    //     if (rootItem) {
                    //         rootItem.close()
                    //     }
                    // }
                }`

            let editorObject = Qt.createQmlObject(aceEditorQml, aceEditorContainer, "aceEditor")
            if (editorObject !== null) {
                root._aceEditorObject = editorObject
                editorObject.rootItem = root
                
                editorObject.editorReady.connect(function() {
                    root._setEditorKeybindings()
                    root._aceEditorObject.setEditorOption("fontSize", root.fontSize)
                    root._aceEditorObject.content = root.text
                    root._aceEditorObject.mode = root._aceMode
                    root._aceEditorObject.theme = "tomorrow_night"
                })
            } else {
                console.log("Ace editor not available: failed to create")
            }
        }
    }

    Rectangle {
        id: simpleEditorBorder
        anchors.top: topBar.bottom
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        visible: root.text !== "" && (root._aceEditorObject === null || root._useSimpleCodeEditor)
        color: Theme.backgroundColorLight
        border.width: 1
        border.color: Theme.borderColor

        Loader {
            id: textEditorLoader
            anchors.fill: parent
            anchors.margins: 1
            sourceComponent: (root._aceEditorObject === null || root._useSimpleCodeEditor) ? textEditorComponent : null

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
        visible: root._aceEditorObject === null && !root._useSimpleCodeEditor
        anchors.bottom: parent.bottom
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.spacingLoose
        wrapMode: Text.Wrap
        horizontalAlignment: Text.AlignHCenter
        color: Theme.textColor || "red"
        text: "Integrated code editor could not be loaded. Qt web engine or code editor QML component is likely missing.\n"+
              "You are using a simple text editor instead. To continue using simple editor without this warning, choose \"internal (simple)\" in settings."
    }

    Shortcut {
        sequence: StandardKey.Save
        onActivated: {
            root.save()
        }
    }

    function _updateUseSimpleCodeEditor() {
        let preferences = LK.config.getPreferences()
        let textEditor = preferences.textEditor
        root._useSimpleCodeEditor = textEditor === "internal-simple"
    }

    function _loadEditorPreferences() {
        let preferences = LK.config.getPreferences()
        if (preferences.editorPreferences) {
            if (preferences.editorPreferences.editMode) {
                root.editMode = preferences.editorPreferences.editMode
            }
            if (preferences.editorPreferences.fontSize) {
                root.fontSize = preferences.editorPreferences.fontSize
            }
        }
    }

    function _saveEditorPreferences() {
        let preferences = LK.config.getPreferences()
        preferences.editorPreferences = preferences.editorPreferences
        preferences.editorPreferences.editMode = root.editMode
        preferences.editorPreferences.fontSize = root.fontSize
        LK.config.setPreferences(preferences)
    }

    function _setEditorKeybindings() {
        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            let handlerPath = null
            if (root.editMode === "vim") {
                handlerPath = "ace/keyboard/vim"
            } else if (root.editMode === "emacs") {
                handlerPath = "ace/keyboard/emacs"
            }

            root._aceEditorObject.callEditorFunction("setKeyboardHandler", handlerPath)
        }
    }

    function save() {
        if (root.commandId === "" || root.localFilePath === "") {
            return
        }

        if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
            root._aceEditorObject.getContent(function(content) {
                root.saved(root.commandId, root.localFilePath, content)
            })
        } else if (root.textEditorItem) {
            root.textEditorItem.save()
        }
    }

    function activate() {
        root._updateUseSimpleCodeEditor()
        
        if (root.pendingInvocation === 0) {
            if (root._aceEditorObject !== null && !root._useSimpleCodeEditor) {
                root._aceEditorObject.getContent(function(content) {
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
}
