import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "Misc"
import "Text"


ToolBar {
    id: root
    property int errorCount: 0
    property int jobsLeft: 0
    padding: 0

    background: BorderRectangle {
        backgroundColor: Theme.backgroundColor
        borderColor: Theme.borderColor
        borderTop: 1
    }

    RowLayout {
        height: errorCountText.implicitHeight

        NormalText {
            id: errorCountText
            text: "Error Count: " + root.errorCount
        }

        NormalText {
            text: "Jobs Left: " + root.jobsLeft
        }
    }
}