extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration::Configuration;


#[derive(QObject, Default)]
pub struct ConfigManagerModel {
    base: qt_base_class!(trait QObject),

    configuration: Configuration,
}

impl ConfigManagerModel {
    pub fn new(configuration: Configuration) -> ConfigManagerModel {
        ConfigManagerModel {
            configuration: configuration,
            ..Default::default()
        }
    }
}