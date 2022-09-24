
function confirmationDialog(parent, text, onAccepted) {
    _create(parent, "ConfirmationDialog", { "text": text }, { "onAccepted": onAccepted })
}


var _dynamicComponents = {
    "ConfirmationDialog": {
        qmlPath: "../ConfirmationDialog.qml",
        defaultProperties: {
            // x: 100,
            // y: 100,
            text: "",
        },
        component: null,
        // TODO: clean up destroyed objects?
        instances: [],
    }
}

function _create(parent, componentId, userProperties, signalHandlers) {
    console.log("Creating new UI object for " + componentId)

    let data = _dynamicComponents[componentId]
    if (data.component === null) {
        data.component = Qt.createComponent(data.qmlPath)
    }

    var properties = Object.assign(data.defaultProperties, userProperties)
    if (data.component.status === Component.Ready) {
        _finishCreation(parent, data, properties, signalHandlers)
    }
    else {
        data.component.statusChanged.connect(() => _finishCreation(parent, data, properties, signalHandlers))
    }
}

function _finishCreation(parent, data, properties, signalHandlers) {
    if (data.component.status === Component.Ready) {
        var instance = data.component.createObject(parent, properties)

        if (instance !== null) {
            for (const [name, handler] of Object.entries(signalHandlers)) {
                instance[name].connect(handler)
            }

            data.instances.push(instance)
            console.log("New object created successfully")
        }
        else {
            _error()
        }
    }
    else if (data.component.status === Component.Error) {
        _error()
    }
}

function _error() {
    console.log("Error creating component")
}
