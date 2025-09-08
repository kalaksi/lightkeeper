var ANSI_COLORS = {
    "30": "black",
    "31": "red",
    "32": "green",
    "33": "yellow",
    "34": "blue",
    "35": "magenta",
    "36": "cyan",
    "37": "white",
    "90": "grey",
}

function trimNewline(text) {
    if (text.endsWith('\n')) {
        return text.slice(0, -1)
    }
    else {
        return text
    }
}

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

/// Parse ANSI color codes and return rich text with appropriate coloring.
function ansiToRichText(text) {

    return text.replace(/\[1?\;?(1|3[0-7]|90|0)m/g, (match, p1) => {
        if (p1 in ANSI_COLORS) {
            return `<span style="color:${ANSI_COLORS[p1]}">`
        }
        else if (p1 === "1") {
            return '<span style="font-weight:bold">'
        }
        else if (p1 === "0") {
            return "</span>"
        }
        else {
            return match
        }
    })
}