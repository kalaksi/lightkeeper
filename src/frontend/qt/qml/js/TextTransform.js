

function capitalize(text) {
    if (text.length > 1) {
        return text[0].toUpperCase() + text.slice(1)
    }
    else {
        return text
    }
}

function truncate(text, maxLength) {
    if (text.length > maxLength) {
        return text.substr(0, maxLength-1) + '…'
    }
    return text
}

function removeWhitespaces(text) {
    return text.replace(/\s/g, '')
}