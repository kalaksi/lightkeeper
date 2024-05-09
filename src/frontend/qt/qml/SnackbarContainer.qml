import QtQuick 2.15

import "js/Utils.js" as Utils


/// Provides a space for multiple snackbars.
Item {
    id: root
    property int spacing: 10
    property int showDuration: 5000
    property int animationDuration: Theme.animationDuration
    property int snackbarHeight: 80
    property int snackbarMaximumWidth: 600
    property var _instances: []

    Component {
        id: snackbarComponent

        Snackbar {
            Behavior on y {
                NumberAnimation {
                    duration: root.animationDuration
                }
            }
        }
    }

    Timer {
        id: cleanupTimer 
        interval: 200
        onTriggered: {
            // Destroy oldest snackbar.
            if (root._instances.length > 0 &&
                Date.now() - root._instances[0].creationTime > root.showDuration) {

                root._instances[0].destroy()
                root._instances.splice(0, 1)
            }

            if (root._instances.length > 0) {
                start()
            }
        }
    }

    function addSnackbar(criticality, text) {
        let newSnackbar = snackbarComponent.createObject(root, {
            criticality: criticality,
            text: text,
            fadeDuration: root.animationDuration,
            showDuration: root.showDuration,
            maximumWidth: root.snackbarMaximumWidth,
            height: root.snackbarHeight,
            "anchors.right": root.right,
            y: root.height - root.snackbarHeight - root.spacing,
        })

        for (let existingInstance of root._instances) {
            existingInstance.y -= newSnackbar.height + root.spacing
        }

        root._instances.push(newSnackbar)
        cleanupTimer.start()
    }
}