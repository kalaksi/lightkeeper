import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11


// This component should be a direct child of main window.
Dialog {
    id: root
    property string explanationText: ""
    property var inputSpecs: []
    property string _errorText: ""
    modal: true
    implicitWidth: 400
    implicitHeight: 300
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal inputValuesGiven(var inputValues)

    background: DialogBackground { }

    contentItem: ColumnLayout {
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
                }

                TextField {
                    text: modelData.default_value
                    validator: RegularExpressionValidator {
                        regularExpression: modelData.validator_regexp === "" ? /.*/ : RegExp(modelData.validator_regexp)
                    }

                    Layout.fillWidth: true
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
            let nextValue = inputRepeater.itemAt(i).children[1].text
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

            // Reset field back to default value.
            inputRepeater.itemAt(i).children[1].text = root.inputSpecs[i].default_value
        }

        root._errorText = ""
        root.inputValuesGiven(values)
    }
}