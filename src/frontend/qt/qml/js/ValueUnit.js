
function AsText(value, unit, empty = "") {
    if (typeof value === "undefined" || value === null || value.length === 0) {
        return empty
    }
    else {
        return value + " " + unit
    }
}