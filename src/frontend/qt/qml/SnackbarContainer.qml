import QtQuick 2.15

import "js/Utils.js" as Utils


/// Provides a space for multiple snackbars.
Item {
    id: root
    property int spacing: 10
    property int showDuration: Theme.notification_show_duration()
    property int animationDuration: Theme.animation_duration()
    property int snackbarHeight: 75
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
        // Has to give time for the fade out to happen. Seems to require some extra time.
        interval: root.showDuration + root.animationDuration + 100
        onTriggered: destroyOldestSnackbar()
        repeat: root._instances.length > 0
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

    function destroyOldestSnackbar() {
        if (root._instances.length > 0) {
            root._instances[0].destroy()
            root._instances.splice(0, 1)
        }
    }
}