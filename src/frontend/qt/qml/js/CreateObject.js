
function confirmationDialog(parent, text, onAccepted) {
    return create(parent, "ConfirmationDialog", { "text": text }, { "onAccepted": onAccepted })
}

function detailsDialog(parent, text, errorText, criticality) {
    return create(parent, "DetailsDialog", { "text": text, "errorText": errorText, "criticality": criticality }, {})
}

function create(parent, componentId, userProperties, signalHandlers) {
    console.log("Creating new UI object for " + componentId)

    let data = _dynamicComponents[componentId]
    if (data.component === null) {
        data.component = Qt.createComponent(data.qmlPath)
    }

    data.lastInstanceId += 1
    var properties = Object.assign(data.defaultProperties, userProperties)
    if (data.component.status === Component.Ready) {
        _finishCreation(parent, data, properties, signalHandlers)
    }
    else {
        data.component.statusChanged.connect(() => _finishCreation(parent, data, properties, signalHandlers))
    }

    return data.lastInstanceId
}

function get(componentId, instanceId) {
    let instance = _dynamicComponents[componentId].instances[instanceId]
    if (typeof instance === "undefined") {
        console.log(`Object ${componentId}:${instanceId} does not exist or is not ready yet`)
    }
    return instance
}

// TODO: separate files (and a "type") if there's going to be more dynamic objects.
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
        instances: {},
        lastInstanceId: 0,
    },
    "DetailsDialog": {
        qmlPath: "../DetailsDialog.qml",
        defaultProperties: { },
        component: null,
        instances: {},
        lastInstanceId: 0,
    }
}

function _finishCreation(parent, data, properties, signalHandlers) {
    if (data.component.status === Component.Ready) {
        var instance = data.component.createObject(parent, properties)

        if (instance !== null) {
            for (const [name, handler] of Object.entries(signalHandlers)) {
                instance[name].connect(handler)
            }

            data.instances[data.lastInstanceId] = instance
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
