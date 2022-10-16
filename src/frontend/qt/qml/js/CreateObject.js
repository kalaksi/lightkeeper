
function detailsDialog(parent, text, errorText, criticality) {
    return create(parent, "DetailsDialog", { "text": text, "errorText": errorText, "criticality": criticality }, {})
}

function create(parent, componentId, userProperties, signalHandlers) {
    console.log("Creating new UI object for " + componentId)

    let data = _dynamicComponents[componentId]
    if (data.component === null) {
        data.component = Qt.createComponent(data.qmlPath)
    }

    let instanceId = _instances.length
    var properties = Object.assign(data.defaultProperties, userProperties)
    if (data.component.status === Component.Ready) {
        _finishCreation(parent, data, properties, signalHandlers)
    }
    else {
        data.component.statusChanged.connect(() => _finishCreation(parent, data, properties, signalHandlers))
    }

    return instanceId
}

function get(instanceId) {
    let instance = _instances[instanceId]
    if (typeof instance === "undefined") {
        console.log(`Object ${instanceId} does not exist or is not ready yet`)
    }
    return instance
}

// TODO: separate files (and a "type") if there's going to be more dynamic objects.
let _dynamicComponents = {
    "DetailsDialog": {
        qmlPath: "../DetailsDialog.qml",
        defaultProperties: { },
        component: null,
    },
}

// TODO: clean up destroyed objects?
let _instances = []

function _finishCreation(parent, data, properties, signalHandlers) {
    if (data.component.status === Component.Ready) {
        var instance = data.component.createObject(parent, properties)

        if (instance !== null) {
            for (const [name, handler] of Object.entries(signalHandlers)) {
                instance[name].connect(handler)
            }

            _instances.push(instance)
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
