

function capitalize(text) {
    if (typeof text[0] === "undefined") {
        return undefined
    }
    return text[0].toUpperCase() + text.slice(1)
}

function truncate(text, maxLength) {
    if (text.length > maxLength) {
        return text.substr(0, maxLength-1) + '…'
    }
    return text
}