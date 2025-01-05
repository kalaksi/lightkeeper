import QtQuick


Item {
    id: root
    default required property Component component

    property var _instances: []
    property int _currentInstanceId: 1

    function create(properties = {}, signalHandlers = {}) {
        let instanceId = _currentInstanceId
        _currentInstanceId += 1

        let instance = component.createObject(root, properties)
        if (instance !== null) {
            _instances[instanceId] = instance
            for (const [name, handler] of Object.entries(signalHandlers)) {
                instance[name].connect(handler)
            }
        }
        else {
            console.log("Error creating object")
        }

        return [instanceId, instance]
    }

    function get(instanceId) {
        let instance = root._instances[instanceId]
        if (typeof instance === "undefined") {
            console.log(`Object ${instanceId} does not exist or is not ready yet`)
        }
        return instance
    }

    function destroyInstance(instanceId) {
        let instance = root.get(instanceId)
        if (typeof instance !== "undefined") {
            // console.log("Destroying instance " + instanceId)
            instance.destroy()
            delete root._instances[instanceId]
        }
    }

    function clear() {
        for (let instance of root._instances) {
            instance.destroy()
        }
        root._instances = []
        root._currentInstanceId = 1
    }
}