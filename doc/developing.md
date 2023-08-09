# Conventions
- When naming command or monitoring modules, prefer familiar terminology and naming from the underlying program.

# Limitations
## qmetaobject (rust crate)
It doesn't seem possible to return custom QML objects from rust to QML, so that QML could access the properties.  
It is possible to pass custom QObjects *through* QML (i.e. rust -> QML -> rust) using QMetaType **or** instantiate custom QML types in QML side.

This limitation is the reason there may, in some cases, be a lot of function calls that return simple values instead of complex objects or they return objects as JSON strings.
