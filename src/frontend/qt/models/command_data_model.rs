extern crate qmetaobject;
use qmetaobject::*;

use crate::frontend;

#[derive(Default, Clone)]
pub struct CommandDataModel {
    pub data: QVariantList,
}

impl CommandDataModel {
    pub fn new(host_display_data: &frontend::HostDisplayData) -> Self {
        let mut model = CommandDataModel {
            data: QVariantList::default(),
        };

        for (_, data) in &host_display_data.command_data {
            model.data.push(serde_json::to_string(&data).unwrap().to_qvariant());
        }

        model
    }
}


impl QMetaType for CommandDataModel {

}