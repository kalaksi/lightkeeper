# Basic project information
- Rust-project with QML frontend.
- Ignore CLI frontend.
- When being asked a question, don't implement anything without confirming first.

# Implementation and style guidelines
- Never change Cargo.toml without explicit approval.
- Don't add trivial comments.
- Always have an empty line before start of a QML element, e.g. `\nText {...`.
- Avoid writing lines that are over 120 characters long.

### QML
- Component structure is: "id: root", public properties, private properties, signals, handlers/slots, contents/children, private functions, public functions
- Reference errors to `LK` are expected and can be ignored.
- When adding or removing QML files, resource file has to be updated.
