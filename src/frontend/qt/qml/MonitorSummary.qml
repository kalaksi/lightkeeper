
import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

Item {
    id: root
    required property string monitorDataJson

    property var _monitorData: []

    Component.onCompleted: function() {
        // console.log(JSON.stringify(monitorDataJson))
        console.log(monitorDataJson)
    }

    Row {
        Text {
            // text: root._monitorData[0]
            // text: root.monitorDataJson
            // text: getValue(monitorDataJson)
            // text: monitorDataJson.data
            text: monitorDataJson
        }
        /*
        Repeater {
            model: root.monitorDataJson

            Text {
                // text: getText(index)
                text: "asdf" + modelData
            }
        }
        */
    }

    function getValue(jsonData) {
        // console.log(jsonData[0])
        // console.log(JSON.stringify(jsonData.length))
        console.log(JSON.stringify(monitorDataJson))
        return "asdf"
    }

    function getText(index) {
        console.log(JSON.stringify(monitorDataJson))

        if (monitorDataJson.length > index) {
            // console.log("adsf")
            // let values = JSON.parse(monitorDataJson)
            //return values[index]
        }

        return "{}"
    }
}