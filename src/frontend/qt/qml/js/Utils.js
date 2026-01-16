function clamp(value, min, max) {
    return Math.min(Math.max(value, min), max);
}

function isIpv4OrIpv6Address(address) {
    let expression = /((^\s*((([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5]))\s*$)|(^\s*((([0-9A-Fa-f]{1,4}:){7}([0-9A-Fa-f]{1,4}|:))|(([0-9A-Fa-f]{1,4}:){6}(:[0-9A-Fa-f]{1,4}|((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})|:))|(([0-9A-Fa-f]{1,4}:){5}(((:[0-9A-Fa-f]{1,4}){1,2})|:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3})|:))|(([0-9A-Fa-f]{1,4}:){4}(((:[0-9A-Fa-f]{1,4}){1,3})|((:[0-9A-Fa-f]{1,4})?:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){3}(((:[0-9A-Fa-f]{1,4}){1,4})|((:[0-9A-Fa-f]{1,4}){0,2}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){2}(((:[0-9A-Fa-f]{1,4}){1,5})|((:[0-9A-Fa-f]{1,4}){0,3}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(([0-9A-Fa-f]{1,4}:){1}(((:[0-9A-Fa-f]{1,4}){1,6})|((:[0-9A-Fa-f]{1,4}){0,4}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:))|(:(((:[0-9A-Fa-f]{1,4}){1,7})|((:[0-9A-Fa-f]{1,4}){0,5}:((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}))|:)))(%.+)?\s*$))/;
    return expression.test(address);
}

function sortNumerically(array) {
    return array.sort((a, b) => a - b);
}

/// Returns time zone in format +HH:MM or -HH:MM
function formatTimezone(timezoneOffsetString) {
    let timezoneOffset = parseInt(timezoneOffsetString);
    let sign = timezoneOffset < 0 ? '-' : '+';
    let hours = Math.abs(Math.floor(timezoneOffset / 60));
    let minutes = Math.abs(timezoneOffset % 60);
    return sign + (hours < 10 ? '0' : '') + hours + ':' + (minutes < 10 ? '0' : '') + minutes;
}

function getLocalTimezoneISOString(now) {
    let date = now === undefined ? new Date() : new Date(now);
    const year = date.getFullYear();
    const month = ('0' + (date.getMonth() + 1)).slice(-2); // Months are zero-based
    const day = ('0' + date.getDate()).slice(-2);
    const hours = ('0' + date.getHours()).slice(-2);
    const minutes = ('0' + date.getMinutes()).slice(-2);
    const seconds = ('0' + date.getSeconds()).slice(-2);

    return year + '-' + month + '-' + day + 'T' + hours + ':' + minutes + ':' + seconds;
}

/// Detects language definition name for SyntaxHighlighter based on file path.
/// Returns language name string or empty string if unknown.
function detectLanguageFromPath(filePath) {
    if (!filePath) {
        return ""
    }

    // Extract file extension
    let lastDot = filePath.lastIndexOf('.')
    if (lastDot === -1 || lastDot === filePath.length - 1) {
        return ""
    }

    let extension = filePath.substring(lastDot + 1).toLowerCase()

    // Locally defined files use hash suffix, so removing it.
    if (extension.includes('_')) {
        extension = extension.substring(0, extension.indexOf('_'))
    }

    // Map common file extensions to SyntaxHighlighter language definitions
    let extensionMap = {
        // Scripting languages
        "sh": "Bash",
        "bash": "Bash",
        "zsh": "Bash",
        "py": "Python",
        "pyw": "Python",
        "rb": "Ruby",
        "pl": "Perl",
        "php": "PHP",
        "lua": "Lua",
        "js": "JavaScript",
        "jsx": "JavaScript",
        "ts": "TypeScript",
        "tsx": "TypeScript",
        "mjs": "JavaScript",
        "cjs": "JavaScript",

        "conf": "INI Files",
        "ini": "INI Files",
        "cfg": "INI Files",
        "config": "INI Files",
        "yaml": "YAML",
        "yml": "YAML",
        "toml": "TOML",
        "json": "JSON",
        "xml": "XML",
        "html": "HTML",
        "htm": "HTML",
        "xhtml": "HTML",

        // Compiled languages
        "c": "C",
        "h": "C",
        "cpp": "C++",
        "cxx": "C++",
        "cc": "C++",
        "hpp": "C++",
        "hxx": "C++",
        "java": "Java",
        "cs": "C#",
        "go": "Go",
        "rs": "Rust",
        "swift": "Swift",
        "kt": "Kotlin",
        "scala": "Scala",

        // Markup and documentation
        "md": "Markdown",
        "markdown": "Markdown",
        "rst": "reStructuredText",
        "tex": "LaTeX",
        "css": "CSS",
        "scss": "SCSS",
        "sass": "SASS",
        "less": "LESS",

        // Shell and system files
        "dockerfile": "Dockerfile",
        "makefile": "Makefile",
        "mk": "Makefile",
        "cmake": "CMake",
        "cmakecache": "CMake",

        // Data formats
        "sql": "SQL",
        "csv": "CSV",
        "diff": "Diff",
        "patch": "Diff",

        // Other
        "log": "Log File",
        "txt": "",
        "text": "",
    }

    return extensionMap[extension] || ""
}