
import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    required property var values

    Text {
        text: getText()
    }

    function getText() {
        if (typeof values === "undefined") {
            return "asdf"
        }
        else {
            return values.join(" i ")
        }
    }
}