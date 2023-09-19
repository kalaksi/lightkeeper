import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "../StyleOverride"
import "../Text"


// This component should be a direct child of main window.
Dialog {
    id: root
    // See UserInputField from rust-side for input spec format.
    property var inputSpecs: []
    property string _errorText: ""
    modal: true
    implicitWidth: 400
    implicitHeight: rootColumn.implicitHeight + Theme.margin_dialog_bottom()
    standardButtons: Dialog.Ok | Dialog.Cancel
    background: DialogBackground { }

    signal inputValuesGiven(var inputValues)


    contentItem: ColumnLayout {
        id: rootColumn
        anchors.fill: parent
        anchors.margins: 30

        Repeater {
            id: inputRepeater
            model: root.inputSpecs

            RowLayout {
                Layout.fillWidth: true

                Label {
                    text: modelData.label
                    Layout.fillWidth: true
                    Layout.alignment: Qt.AlignTop
                }

                TextField {
                    visible: modelData.field_type !== "Option"
                    text: modelData.default_value || ""
                    validator: RegularExpressionValidator {
                        regularExpression: modelData.validator_regexp === "" ? /.*/ : RegExp(modelData.validator_regexp)
                    }
                    Layout.fillWidth: true
                }

                Column {
                    visible: modelData.field_type === "Option"
                    spacing: Theme.spacing_normal()

                    Layout.minimumWidth: 220

                    ComboBox {
                        id: comboBox
                        model: [''].concat(modelData.options)
                        currentIndex: 0
                    }

                    SmallText {
                        width: parent.width
                        text: [''].concat(modelData.option_descriptions)[comboBox.currentIndex]
                        color: Theme.color_dark_text()
                        wrapMode: Text.WordWrap
                    }
                }
            }
        }

        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }

        Label {
            text: root._errorText
            color: "red"
        }
    }

    onAccepted: {
        let values = []
        for (let i = 0; i < root.inputSpecs.length; i++) {
            // Handle options differently.
            let nextValue = ""
            if (root.inputSpecs[i].field_type === "Option") {
                nextValue = inputRepeater.itemAt(i).children[2].children[0].currentText
            }
            else {
                nextValue = inputRepeater.itemAt(i).children[1].text
            }
            values.push(nextValue)

            // For some reason the validator fails to perform correctly in all cases.
            // Here we make sure no invalid values get passed.
            let validator = RegExp(root.inputSpecs[i].validator_regexp)

            // Additional validator is optional.
            let additionalValidator = RegExp(root.inputSpecs[i].additional_validator_regexp)

            if (!validator.test(nextValue) || 
                (root.inputSpecs[i].additional_validator_regexp !== "" && !additionalValidator.test(nextValue))) {

                console.log(`Invalid value for "${root.inputSpecs[i].label}": ${nextValue}`)
                root._errorText = "Invalid value"
                root.open()
                return
            }
        }

        root.inputValuesGiven(values)
        reset()
    }

    onRejected: {
        reset()
    }

    function reset() {
        root._errorText = ""
        root.inputSpecs = []
    }
}