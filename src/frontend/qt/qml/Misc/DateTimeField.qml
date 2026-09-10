/*
 * SPDX-FileCopyrightText: Copyright (C) 2026 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

import Lightkeeper 1.0

import "../Button"
import "../Text"
import "../StyleOverride"


RowLayout {
    id: root

    property alias text: textField.text
    property alias placeholderText: textField.placeholderText
    property alias placeholderTextColor: textField.placeholderTextColor
    property real fieldWidth: 160
    /// Time selecting style in the popup: "numeric" (default) or "tumbler".
    property string timeInputStyle: "numeric"

    signal accepted()
    /// Emitted when the user applies a value from the calendar popup.
    signal picked()

    spacing: Theme.spacingTight

    TextField {
        id: textField
        Layout.preferredWidth: root.fieldWidth
        onAccepted: root.accepted()
    }

    ImageButton {
        id: calendarButton
        imageSource: "qrc:/main/images/button/view-calendar"
        size: textField.implicitHeight * 0.8
        tooltip: "Pick date and time"
        enabled: root.enabled
        onClicked: root.openPicker()

        Layout.preferredWidth: textField.implicitHeight
        Layout.alignment: Qt.AlignVCenter
    }

    Popup {
        id: pickerPopup
        parent: Overlay.overlay
        modal: true
        focus: true
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        padding: Theme.spacingLoose
        leftPadding: Theme.spacingLoose * 2
        rightPadding: Theme.spacingLoose * 2

        onAboutToShow: root.positionPopup()
        onOpened: root.positionPopup()

        background: Rectangle {
            color: Theme.backgroundColor
            border.width: 1
            border.color: Theme.borderColor
            radius: 2
        }

        contentItem: ColumnLayout {
            spacing: Theme.spacingNormal

            RowLayout {
                Layout.fillWidth: true

                ToolButton {
                    text: "<"
                    onClicked: root.shiftMonth(-1)
                }

                NormalText {
                    text: monthGrid.title
                    horizontalAlignment: Text.AlignHCenter
                    Layout.fillWidth: true
                }

                ToolButton {
                    text: ">"
                    onClicked: root.shiftMonth(1)
                }
            }

            DayOfWeekRow {
                locale: monthGrid.locale
                Layout.fillWidth: true

                delegate: SmallText {
                    required property string shortName
                    text: shortName
                    horizontalAlignment: Text.AlignHCenter
                }
            }

            MonthGrid {
                id: monthGrid
                locale: Qt.locale()
                Layout.fillWidth: true

                property date selectedDate: new Date()

                onClicked: function(date) {
                    monthGrid.selectedDate = date
                }

                delegate: SmallText {
                    required property var model
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    opacity: model.month === monthGrid.month ? 1 : 0.4
                    text: model.day
                    font.bold: model.today
                    color: {
                        let selected = monthGrid.selectedDate
                        if (selected.getFullYear() === model.year
                            && selected.getMonth() === model.month
                            && selected.getDate() === model.day) {
                            return Theme.highlightColor
                        }
                        return Theme.textColor
                    }
                }
            }

            RowLayout {
                visible: root.timeInputStyle === "numeric"
                Layout.alignment: Qt.AlignHCenter
                spacing: Theme.spacingLoose

                ColumnLayout {
                    spacing: Theme.spacingTight

                    SmallText {
                        text: "Hour"
                        Layout.alignment: Qt.AlignHCenter
                    }

                    SpinBox {
                        id: hourSpin
                        from: 0
                        to: 23
                        wrap: true
                        editable: true
                        Layout.alignment: Qt.AlignHCenter

                        textFromValue: function(value, locale) {
                            return value < 10 ? "0" + value : "" + value
                        }
                        valueFromText: function(text, locale) {
                            return parseInt(text, 10)
                        }
                    }
                }

                ColumnLayout {
                    spacing: Theme.spacingTight

                    SmallText {
                        text: "Min"
                        Layout.alignment: Qt.AlignHCenter
                    }

                    SpinBox {
                        id: minuteSpin
                        from: 0
                        to: 59
                        wrap: true
                        editable: true
                        Layout.alignment: Qt.AlignHCenter

                        textFromValue: function(value, locale) {
                            return value < 10 ? "0" + value : "" + value
                        }
                        valueFromText: function(text, locale) {
                            return parseInt(text, 10)
                        }
                    }
                }

                ColumnLayout {
                    spacing: Theme.spacingTight

                    SmallText {
                        text: "Sec"
                        Layout.alignment: Qt.AlignHCenter
                    }

                    SpinBox {
                        id: secondSpin
                        from: 0
                        to: 59
                        wrap: true
                        editable: true
                        Layout.alignment: Qt.AlignHCenter

                        textFromValue: function(value, locale) {
                            return value < 10 ? "0" + value : "" + value
                        }
                        valueFromText: function(text, locale) {
                            return parseInt(text, 10)
                        }
                    }
                }
            }

            RowLayout {
                visible: root.timeInputStyle === "tumbler"
                Layout.alignment: Qt.AlignHCenter
                spacing: Theme.spacingLoose

                ColumnLayout {
                    spacing: Theme.spacingTight

                    SmallText {
                        text: "Hour"
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Tumbler {
                        id: hourTumbler
                        model: 24
                        wrap: true
                        visibleItemCount: 5
                        Layout.preferredHeight: 120
                        Layout.preferredWidth: 48

                        delegate: tumblerDelegate
                    }
                }

                ColumnLayout {
                    spacing: Theme.spacingTight

                    SmallText {
                        text: "Min"
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Tumbler {
                        id: minuteTumbler
                        model: 60
                        wrap: true
                        visibleItemCount: 5
                        Layout.preferredHeight: 120
                        Layout.preferredWidth: 48

                        delegate: tumblerDelegate
                    }
                }

                ColumnLayout {
                    spacing: Theme.spacingTight

                    SmallText {
                        text: "Sec"
                        Layout.alignment: Qt.AlignHCenter
                    }

                    Tumbler {
                        id: secondTumbler
                        model: 60
                        wrap: true
                        visibleItemCount: 5
                        Layout.preferredHeight: 120
                        Layout.preferredWidth: 48

                        delegate: tumblerDelegate
                    }
                }
            }

            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: Theme.spacingNormal

                Button {
                    text: "Cancel"
                    flat: true
                    onClicked: pickerPopup.close()
                }

                Button {
                    text: "OK"
                    onClicked: root.applyPicker()
                }
            }
        }
    }

    Component {
        id: tumblerDelegate

        SmallText {
            // Required for Tumbler attached properties with ComponentBehavior: Bound.
            required property int index
            required property int modelData

            text: modelData < 10 ? "0" + modelData : "" + modelData
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            opacity: 0.4 + Math.max(0, 1 - Math.abs(Tumbler.displacement)) * 0.6
        }
    }

    function openPicker() {
        let date = root.parseDateTime(textField.text)
        monthGrid.year = date.getFullYear()
        monthGrid.month = date.getMonth()
        monthGrid.selectedDate = date
        root.setTimeValues(date.getHours(), date.getMinutes(), date.getSeconds())
        pickerPopup.open()
    }

    function setTimeValues(hours, minutes, seconds) {
        hourSpin.value = hours
        minuteSpin.value = minutes
        secondSpin.value = seconds
        hourTumbler.currentIndex = hours
        minuteTumbler.currentIndex = minutes
        secondTumbler.currentIndex = seconds
    }

    function positionPopup() {
        let overlay = pickerPopup.parent
        if (!overlay) {
            return
        }

        let below = calendarButton.mapToItem(overlay, 0, calendarButton.height)
        let x = below.x
        let y = below.y + Theme.spacingTight

        if (pickerPopup.width > 0 && x + pickerPopup.width > overlay.width - Theme.spacingNormal) {
            x = overlay.width - pickerPopup.width - Theme.spacingNormal
        }
        if (x < Theme.spacingNormal) {
            x = Theme.spacingNormal
        }

        if (pickerPopup.height > 0 && y + pickerPopup.height > overlay.height - Theme.spacingNormal) {
            let above = calendarButton.mapToItem(overlay, 0, 0)
            y = above.y - pickerPopup.height - Theme.spacingTight
        }
        if (y < Theme.spacingNormal) {
            y = Theme.spacingNormal
        }

        pickerPopup.x = Math.round(x)
        pickerPopup.y = Math.round(y)
    }

    function shiftMonth(delta) {
        let month = monthGrid.month + delta
        let year = monthGrid.year
        if (month < 0) {
            month = 11
            year -= 1
        }
        else if (month > 11) {
            month = 0
            year += 1
        }
        monthGrid.month = month
        monthGrid.year = year
    }

    function applyPicker() {
        let date = monthGrid.selectedDate
        let year = date.getFullYear()
        let month = ("0" + (date.getMonth() + 1)).slice(-2)
        let day = ("0" + date.getDate()).slice(-2)
        let hourValue = root.timeInputStyle === "tumbler" ? hourTumbler.currentIndex : hourSpin.value
        let minuteValue = root.timeInputStyle === "tumbler" ? minuteTumbler.currentIndex : minuteSpin.value
        let secondValue = root.timeInputStyle === "tumbler" ? secondTumbler.currentIndex : secondSpin.value
        let hours = ("0" + hourValue).slice(-2)
        let minutes = ("0" + minuteValue).slice(-2)
        let seconds = ("0" + secondValue).slice(-2)
        textField.text = year + "-" + month + "-" + day + " " + hours + ":" + minutes + ":" + seconds
        pickerPopup.close()
        root.picked()
    }

    function parseDateTime(value) {
        // Absolute: YYYY-MM-DD HH:MM:SS with optional " UTC" suffix.
        let match = /^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2}):(\d{2})(?: UTC)?$/.exec(value)
        if (match) {
            return new Date(
                parseInt(match[1], 10),
                parseInt(match[2], 10) - 1,
                parseInt(match[3], 10),
                parseInt(match[4], 10),
                parseInt(match[5], 10),
                parseInt(match[6], 10)
            )
        }
        return new Date()
    }
}

