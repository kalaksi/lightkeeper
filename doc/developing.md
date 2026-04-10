# Conventions
- When naming command or monitoring modules, prefer familiar terminology and naming from the underlying program.
- Prefer doing text processing in Lightkeeper, not on target host (awk, sed...).

# Limitations
## qmetaobject (rust crate)
It doesn't seem possible to return custom QML objects from rust to QML, so that QML could access the properties.  
It is possible to pass custom QObjects *through* QML (i.e. rust -> QML -> rust) using QMetaType **or** instantiate custom QML types in QML side.

This limitation is the reason there may, in some cases, be a lot of function calls that return simple values instead of complex objects or they return objects as JSON strings.

# Problems
If QML linter can't find modules, add to project root in `.qmlls.ini` (absolute paths). Use one `importPaths=` entry with directories separated by `;` (Qt convention), for example:
```
[General]
importPaths=PROJECT_PATH/third_party/qml-lighthouse-components;PROJECT_PATH/third_party/ChartJs2QML;PROJECT_PATH/src/frontend/qt/qml_types
```

The `Lightkeeper` QML module lives under `src/frontend/qt/qml_types/Lightkeeper/` (`qmldir` + `plugins.qmltypes`). The import path must be the **parent** of that folder (`…/qml_types`), not `…/qml_types/Lightkeeper`.

Type stubs: `src/frontend/qt/qml_types/Lightkeeper/plugins.qmltypes` (keep in sync with Rust).


