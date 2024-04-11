import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.11

import "Misc"
import "Text"


ToolBar {
    id: root
    required property int errorCount
    required property int jobsLeft
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
            text: "Errors: " + root.errorCount
        }

        NormalText {
            id: jobsText
            rightPadding: Theme.spacingLoose * 2
            text: root.jobsLeft + " jobs left"
            // This makes the text less prominent when there are no jobs left.
            // color: root.jobsLeft > 0 ? Theme.textColor : Theme.disabledTextColor
        }
    }
}