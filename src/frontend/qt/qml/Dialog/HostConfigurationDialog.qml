import QtQuick 2.15
import QtQuick.Controls 1.4
import QtQuick.Controls 2.15
import QtQuick.Controls.Material 2.15
import QtQuick.Layouts 1.11


// This component should be a direct child of main window.
Dialog {
    id: root
    required property string hostName
    required property var hostSettings
    required property var groups
    property int _contentWidth: 350

    modal: true
    implicitWidth: 500
    implicitHeight: 600
    standardButtons: Dialog.Ok | Dialog.Cancel

    contentItem: ColumnLayout {
        id: content
        anchors.fill: parent
        anchors.margins: Theme.dialog_margin()
        spacing: Theme.form_row_spacing()

        Column {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth

            Label {
                text: "Name"
            }

            TextField {
                width: parent.width
                placeholderText: "Unique name for host..."
                text: root.hostSettings !== undefined ? root.hostName: ""
                // TODO: validation
            }
        }

        Column {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth

            Label {
                text: "IP Address or domain name"
            }

            TextField {
                width: parent.width
                placeholderText: ""
                text: {
                    if (root.hostSettings !== undefined) {
                        if (root.hostSettings.address === "0.0.0.0") {
                            return root.hostSettings.fqdn
                        }
                        else {
                            return root.hostSettings.address 
                        }
                    }
                    else {
                        return ""
                    }
                }
            }
        }

        // Just for extra spacing
        Item {
            Layout.fillWidth: true
            height: Theme.form_row_spacing()
        }

        TabView {
            Layout.alignment: Qt.AlignHCenter
            Layout.preferredWidth: root._contentWidth
            Layout.fillHeight: true

            Tab {
                title: "Selected groups"

                ListView {
                    clip: true
                    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                    boundsBehavior: Flickable.StopAtBounds

                    ScrollBar.vertical: ScrollBar {
                        active: true
                    }

                    model: root.hostSettings !== undefined ? root.hostSettings.groups : []
                    delegate: ItemDelegate {
                        width: parent.width
                        text: modelData
                    }
                }
            }

            Tab {
                title: "Available groups"

                ListView {
                    clip: true
                    // TODO: use selectionBehavior etc. after upgrading to Qt >= 6.4
                    boundsBehavior: Flickable.StopAtBounds

                    ScrollBar.vertical: ScrollBar {
                        active: true
                    }

                    model: root.groups
                    delegate: ItemDelegate {
                        width: parent.width
                        text: modelData
                    }
                }
            }
        }

        // Content will overflow behind the buttons with Layout.fillHeight (ugh...), reserve some space with them with this.
        Item {
            Layout.fillWidth: true
            height: 40
        }
    }
}