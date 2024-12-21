extern crate qmetaobject;
use qmetaobject::*;

// QMetaType implementation makes it possible to pass the object to QML and back,
// but it is not possible to otherwise use the object in a QML context.

impl QMetaType for crate::module::monitoring::DataPoint {
}

impl QMetaType for crate::module::monitoring::MonitoringData {
}

impl QMetaType for crate::command_handler::CommandButtonData {
}

impl QMetaType for crate::frontend::DisplayData {
}

impl QMetaType for crate::configuration::DisplayOptions {
}

impl QMetaType for crate::configuration::Configuration {
}

impl QMetaType for crate::configuration::Hosts {
}

impl QMetaType for crate::configuration::Groups {
}