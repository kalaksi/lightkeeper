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
        // If previous errors exist, give some extra time so boxes won't disappear at the same time.
        let extraDisplayMs = root._instances.length * 500

        let newSnackbar = snackbarComponent.createObject(root, {
            creationTime: Date.now() + extraDisplayMs,
            criticality: criticality,
            text: text,
            fadeDuration: root.animationDuration,
            showDuration: root.showDuration + extraDisplayMs,
            maximumWidth: root.snackbarMaximumWidth,
            height: root.snackbarHeight,
            "anchors.right": root.right,
            y: root.height - root.snackbarHeight - root.spacing,
        })

        for (let i = 0; i < root._instances.length; i++) {
            let y_multiplier = root._instances.length - i
            root._instances[i].y = root.height - root.snackbarHeight - root.spacing - (root.snackbarHeight + root.spacing) * y_multiplier
        }

        root._instances.push(newSnackbar)
        cleanupTimer.start()
    }
}