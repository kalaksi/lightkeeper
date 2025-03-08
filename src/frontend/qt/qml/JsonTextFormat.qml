import QtQuick

import "Text"
import "js/Parse.js" as Parse

/// A component that displays JSON text in a markdown-like format.
/// If the text doesn't seem to be JSON, it will be displayed as is.
Item {
    id: root
    required property string jsonText
    implicitHeight: textContent.height
    implicitWidth: textContent.width

    NormalText {
        id: textContent
        wrapMode: Text.WordWrap
        textFormat: Text.MarkdownText
        text: root.createMarkdown(root.jsonText)
    }

    // TODO: Move these to a rust model?
    function createMarkdown(jsonText) {
        let text = ""

        if (jsonText !== "") {
            let data = Parse.TryParseJson(jsonText)

            if (data === null) {
                text += jsonText
            }
            else {
                text += objectToMarkdown(data, 0)
            }
        }

        return text
    }

    function objectToMarkdown(jsObject, indentLevel) {
        let text = ""
        let prefix = "    ".repeat(indentLevel)
        let entries = Object.entries(jsObject)

        entries.forEach(item => {
            text += `${prefix}* ${item[0]}: `

            if (typeof item[1] === "object") {
                text += "\n"

                if (item[1] !== null) {
                    // Recursive call
                    text += objectToMarkdown(item[1], indentLevel + 1)
                }
            }
            else {
                text += `${item[1]}\n`
            }
        })

        return text
    }

}