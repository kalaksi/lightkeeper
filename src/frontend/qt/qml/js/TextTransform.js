var ANSI_COLORS = {
    30: "black",
    31: "red",
    32: "green",
    33: "yellow",
    34: "blue",
    35: "magenta",
    36: "cyan",
    37: "white",
    90: "grey",
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
    return text.replace(/&/g, "&amp;")
               .replace(/</g, "&lt;")
               .replace(/>/g, "&gt;")
               .replace(/"/g, "&quot;")
               .replace(/'/g, "&#039;");
}

/// Parse ANSI color codes and return rich text with appropriate coloring.
function ansiToRichText(text) {
    return text.replace(/\x1b\[([\d;]+)m/g, (match, p1) => {
        let codes = p1.split(";").map(Number)

        if (codes.includes(0)) {
            return "</span>"
        }

        let styles = []
        for (const code of codes) {
            if (code === 1) styles.push("font-weight:bold");
            else if (code === 3) styles.push("font-style:italic");
            else if (code === 4) styles.push("text-decoration:underline");
            else if (code === 9) styles.push("text-decoration:line-through");
            else if (code in ANSI_COLORS) {
                styles.push(`color:${ANSI_COLORS[code]}`)
            }
        }

        return styles.length > 0 ? `<span style="${styles.join(";")}">` : ""
    })
}