# Basic project information
- This is a Rust-project with QML frontend (Qt 6.10).
- Minimum supported Rust is 1.88 (`rust-version` in `Cargo.toml`).
- Ignore CLI frontend.
- When being asked a question, don't implement anything without confirming first.
- Currently Linux-only, but aims to be multi-platform in the future, so that affects e.g. dependency choices.

# Implementation and style guidelines
- Never change Cargo.toml without explicit approval.
- Don't add new trivial comments.
- Don't add new thin functions that could be inlined (under 5 lines or conceptually 1-2 calls and only called from 1-3 places)
- Never remove existing comments, only update.
- Never write lines that are over 120 characters long.
- Aim for minimum amount of code.
- Ask often! For example when there are alternative solutions.
- Don't use shorthands like "idx" instead of "index".
- Try to keep QML models (*_model.rs) lean. UI shouldn't handle a lot of logic.
- Never run tests automatically, ask if you want to run tests.

## When editing existing code
- Don't "improve" adjacent code, comments, or formatting, but architectural changes are ok.
- Match existing style, even if you'd do it differently.
- Only remove imports/variables/functions that YOUR changes made unused.

## When adding new modules
- Read a few similar modules and follow similar patterns with code style.

## When editing tests
- Use simple unwrap() instead of except("simple message").
- Never create temp files or dirs without confirmation. Try to operate in-memory instead.

## QML
- Always have an empty line before start of a QML element, e.g. `\nText {...`.
- Component structure should be: "id: root", public properties, private properties, signals, handlers/slots, contents/children, private functions, public functions
- When adding or removing QML files, resource file has to be updated.
- Prefer using native components from Qt instead of implementing something custom.
- Prefer iterating with for..of or for..in instead of using indexes.
- Avoid overly complicated width or height calculations.
- When warranted, prefer functional style (map, filter, etc.) instead of imperative.
- Prefer Columns and Rows instead of manually calculating positions and sizes.
- `LK`, `DesktopPortal`, and `Theme` are QML singletons in `Lightkeeper`; any QML file that uses them needs `import Lightkeeper 1.0`.
- Always update and keep in sync Rust-based QML type info (`src/frontend/qt/qml_types/Lightkeeper/plugins.qmltypes`)

## Icons
- Icons should be copied from breeze in `vendor/breeze-icons-5.95.0`-dir.
- Use size 22 for icons.

# Web searches
- Qt 6 has had many changes between versions, so it's important that the information is recent, preferably less than 3 years old. That way Qt 6.10 features can be utilized.
