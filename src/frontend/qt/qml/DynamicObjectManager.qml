import QtQuick


Item {
    id: root
    default required property Component component

    // TODO: clean up destroyed objects?
    property var _instances: []
    property int _currentInstanceId: 1

    function create(properties = {}) {
        let instanceId = _currentInstanceId
        _currentInstanceId += 1

        let instance = component.createObject(root, properties)
        if (instance !== null) {
            _instances[instanceId] = instance
            console.log("New object created successfully")
        }
        else {
            console.log("Error creating object")
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

    function destroy(instanceId) {
        let instance = root.get(instanceId)
        if (typeof instance !== "undefined") {
            instance.destroy()
            delete _instances[instanceId]
        }
    }

    function _finishCreation(parent, component, properties, signalHandlers, instanceId) {
        if (component.status === Component.Ready) {
            let instance = component.createObject(parent, properties)
            if (instance !== null) {
                _instances[instanceId] = instance
                for (const [name, handler] of Object.entries(signalHandlers)) {
                    instance[name].connect(handler)
                }
                console.log("New object created successfully")
            }
            else {
                console.log("Error creating object")
            }
        }
        else if (component.status === Component.Error) {
            console.log(`Error creating component: ${component.errorString()}`)
        }
    }
}