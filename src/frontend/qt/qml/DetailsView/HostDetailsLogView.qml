import QtQuick 2.15
import QtQuick.Controls 2.15
import Qt.labs.qmlmodels 1.0
import QtQuick.Layouts 1.15
import QtQuick.Controls.Material 2.15

Item {
    // TODO: loading indicator and indicator (translucent text maybe) for empty results 
    // TODO: indicator for no search results
    id: root
    required property string hostId
    property string commandId: ""
    property string text: ""
    property string errorText: ""
    property string criticality: ""
    property string _unitId: ""


    Rectangle {
        color: Material.background
        anchors.fill: parent
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Theme.spacing_normal()

        Row {
            id: searchBox
            spacing: Theme.spacing_loose()

            Layout.topMargin: Theme.spacing_normal()
            Layout.leftMargin: root.width * 0.30
            Layout.fillWidth: true

            Row {
                spacing: 10
                bottomPadding: 10

                TextField {
                    id: searchField
                    anchors.leftMargin: 30
                    anchors.verticalCenter: parent.verticalCenter
                    width: searchBox.width * 0.55
                    placeholderText: "RegEx search..."
                    color: Material.foreground
                    focus: true

                    onAccepted: logList.search("up", searchField.text)
                }

                // TODO: custom component for buttons (and roundbuttons).
                Button {
                    anchors.leftMargin: 30
                    anchors.verticalCenter: parent.verticalCenter
                    flat: true

                    ToolTip.visible: hovered
                    ToolTip.text: "Search up"

                    onClicked: logList.search("up", searchField.text)

                    Image {
                        width: 0.80 * parent.width
                        height: width
                        anchors.fill: parent
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/search-up"
                    }
                }

                Button {
                    anchors.verticalCenter: parent.verticalCenter
                    flat: true

                    ToolTip.visible: hovered
                    ToolTip.text: "Search down"

                    onClicked: logList.search("down", searchField.text)

                    Image {
                        width: 0.80 * parent.width
                        height: width
                        anchors.fill: parent
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/search-down"
                    }
                }

                Button {
                    anchors.verticalCenter: parent.verticalCenter
                    onClicked: searchRows(searchField.text)

                    ToolTip.visible: hovered
                    ToolTip.text: "Show only matching rows"

                    Image {
                        width: parent.width * 0.8
                        height: width
                        anchors.centerIn: parent
                        source: "qrc:/main/images/button/search"
                    }
                }
            }
        }

        LogList {
            id: logList
            rows: root.text.split("\n")

            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: Theme.spacing_normal()
        }
    }

    Shortcut {
        sequence: StandardKey.Find
        onActivated: searchField.focus = true
    }

    Shortcut {
        sequence: StandardKey.FindNext
        onActivated: logList.search("up", searchField.text)
    }

    Shortcut {
        sequence: StandardKey.FindPrevious
        onActivated: logList.search("down", searchField.text)
    }
}