import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import "../Text"
import "../StyleOverride"


LightkeeperDialog {
    id: root
    // See UserInputField from rust-side for input spec format.
    property var inputSpecs: []
    property string _errorText: ""
    title: "Input"
    modal: true
    implicitWidth: 400
    implicitHeight: rootColumn.implicitHeight + Theme.marginDialogTop + Theme.marginDialogBottom
    standardButtons: Dialog.Ok | Dialog.Cancel

    signal inputValuesGiven(var inputValues)


    // XXX: there's some kind of bug where using ColumnLayout (instead of Column) and Repeater inside
    // causes a segfault when re-using this dialog.
    contentItem: Column {
        id: rootColumn
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: Theme.marginDialog
        anchors.topMargin: Theme.marginDialogTop
        anchors.bottomMargin: Theme.marginDialogBottom
        spacing: Theme.spacingLoose

        Repeater {
            id: inputRepeater
            model: root.inputSpecs

            RowLayout {
                width: rootColumn.width

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
                    spacing: Theme.spacingNormal

                    Layout.minimumWidth: 220

                    ComboBox {
                        id: comboBox
                        width: parent.width
                        model: [''].concat(modelData.options)
                        currentIndex: 0
                    }

                    SmallText {
                        width: parent.width
                        text: [''].concat(modelData.option_descriptions)[comboBox.currentIndex]
                        color: Theme.textColorDark
                        wrapMode: Text.WordWrap
                    }
                }
            }
        }

        Label {
            height: 50
            text: root._errorText
            color: "red"
        }
    }

    onAccepted: {
        let [values, error] = getInputValues()
        if (error === "") {
            root.inputValuesGiven(values)
            resetFields()
        }
        else {
            root._errorText = error
            root.open()
        }
    }

    onRejected: {
        resetFields()
    }

    function getInputValues() {
        let values = []
        let error = ""

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
                error = "Invalid value"
            }
        }

        return [values, error]
    }

    function resetFields() {
        root._errorText = ""
        root.inputSpecs = []
    }
}