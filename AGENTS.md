# Basic project information
- This is a Rust-project with QML frontend (Qt 6.10).
- Ignore CLI frontend.
- When being asked a question, don't implement anything without confirming first.

# Implementation and style guidelines
- Never change Cargo.toml without explicit approval.
- Don't add new trivial comments.
- Don't remove existing comments when refactoring.
- Avoid writing lines that are over 120 characters long.
- Don't add error handling for impossible scenarios.
- Aim for minimum amount of code.
- Ask often, for example when when there are alternative solutions.

## When editing existing code
- Don't "improve" adjacent code, comments, or formatting.
- Match existing style, even if you'd do it differently.
- Only remove imports/variables/functions that YOUR changes made unused.

## QML
- Always have an empty line before start of a QML element, e.g. `\nText {...`.
- Component structure should be: "id: root", public properties, private properties, signals, handlers/slots, contents/children, private functions, public functions
- Reference errors to `LK` are expected and can be ignored.
- When adding or removing QML files, resource file has to be updated.

# Web searches
- Qt 6 has had many changes between versions, so it's important that the information is recent, preferably less than 3 years old. That way Qt 6.10 features can be utilized.
