import QtQuick 2.15
import Qt.labs.qmlmodels 1.0

import CooldownTimerModel 1.0


// Countdown timer for the cooldown canvas animation
Item {
    id: root

    signal triggered()


    CooldownTimerModel {
        id: cooldowns 
    }

    Timer {
        id: timer

        interval: 20
        repeat: true
        running: false
        onTriggered: {
            let cooldownCount = cooldowns.updateCooldowns(interval)
            if (cooldownCount === 0) {
                timer.stop()
            }

            root.triggered()
        }
    }

    function startCooldown(buttonIdentifier, invocationId) {
        cooldowns.startCooldown(buttonIdentifier, invocationId)

        // Does nothing if timer is already running.
        timer.start()
    }

    function finishCooldown(invocationId) {
        cooldowns.finishCooldown(invocationId)
    }

    function getCooldown(buttonIdentifier) {
        return cooldowns.getCooldown(buttonIdentifier)
    }
}