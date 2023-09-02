

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

/// This is probably not completely secure so don't use it for anything important
function escapeHtml(text)
{
    return text.replace(/</g, "&lt;")
               .replace(/>/g, "&gt;")
               .replace(/&/g, "&amp;")
               .replace(/"/g, "&quot;")
               .replace(/'/g, "&#039;");
 }