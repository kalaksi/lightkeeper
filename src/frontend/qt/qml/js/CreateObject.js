
function confirmationDialog() {
    _create("ConfirmationDialog")
}


var _components = {
    "ConfirmationDialog": {
        qmlPath: "../ConfirmationDialog.qml",
        config: { x: 100, y: 100 },
        component: null,
        // TODO: clean up destroyed objects
        instances: [],
    }
}

function _create(componentId, config) {
    console.log("Creating component " + componentId)

    let data = _components[componentId];
    if (data.component === null) {
        data.component = Qt.createComponent(data.qmlPath)
    }

    if (data.component.status === Component.Ready) {
        _finishCreation(applicationWindow, data)
    }
    else {
        confirmationDialogComponent.statusChanged.connect(() => _finishCreation(applicationWindow, data));
    }
}

function _finishCreation(parent, data) {
    if (data.component.status === Component.Ready) {
        var instance = data.component.createObject(parent, data.config)

        if (instance !== null) {
            data.instances.push(instance)
            console.log("Created new component successfully")
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
