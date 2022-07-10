

function capitalize(inputString) {
    if (typeof inputString[0] === "undefined") {
        return undefined
    }
    return inputString[0].toUpperCase() + inputString.slice(1)
}