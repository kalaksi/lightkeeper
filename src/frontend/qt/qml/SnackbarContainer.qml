import QtQuick 2.15
import QtQuick.Controls.Material 2.15


/// Provides a space for multiple snackbars.
Item {
    id: root
    property int spacing: 10
    property int showDuration: Theme.notification_show_duration()
    property int animationDuration: Theme.animation_duration()
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
            contentWidth: 500,
            contentHeight: 50,
            fadeDuration: root.animationDuration,
            showDuration: root.showDuration,
            y: root.height - 70 - root.spacing,
            x: root.width - 530 - root.spacing,
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