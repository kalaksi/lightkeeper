
function test(app) {
    try {
        testCheckInitialStatus(app)
        testSelectHost(app)
    }
    catch (e) {
        return "FAIL: " + e
    }
    return "OK"
}

function testCheckInitialStatus(app) {
    let table = findByName(app.contentItem, "hostTable")

    assert(table.model.rowCount(), 5)
    assert(table.model.getSelectedHostId(), "")
    assert(getTableElement(table, 0, 1), "centos7")
    assert(getTableElement(table, 1, 1), "centos8")
    assert(getTableElement(table, 2, 1), "debian10")
    assert(getTableElement(table, 3, 1), "debian11")
    assert(getTableElement(table, 4, 1), "ubuntu2004")
}

function testSelectHost(app) {
    let table = findByName(app.contentItem, "hostTable")
    table.model.selectedRow = 1

    assert(table.model.getSelectedHostId(), "centos8")

    let details = findByName(app.contentItem, "detailsView")
    assert(detailsView.visible, true)
}

//
// Test helpers for often used operations.
//

function getTableElement(table, row, column) {
    let tableIndex = table.model.index(row, column)
    return table.model.data(tableIndex, 256)
}


//
// Common helper functions.
//
function assert(value, expected) {
    if (value !== expected) {
        let caller = new Error().stack.split("\n")[1].trim()
        throw `${caller}: expected: ${expected}, got: ${value}`
    }
}

function findByName(parent, name) {
    for (let i = 0; i < parent.children.length; i++) {
        let item = parent.children[i]

        if (item.objectName === name) {
            return item
        }
        else {
            let result = findByName(item, name)
            if (result) {
                return result
            }
        }
    }
}