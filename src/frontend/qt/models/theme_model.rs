extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;


#[derive(QObject, Default)]
pub struct ThemeModel {
    base: qt_base_class!(trait QObject),
    group_multivalue: qt_method!(fn(&self) -> QString),
    category_color: qt_method!(fn(&self, category: QString) -> QString),
    category_icon: qt_method!(fn(&self, category: QString) -> QString),
    allow_collapsing_command: qt_method!(fn(&self, command_id: QString) -> QString),

    display_options: configuration::DisplayOptions,
}

impl ThemeModel {
    pub fn new(display_options: configuration::DisplayOptions) -> ThemeModel {
        ThemeModel {
            display_options: display_options,
            ..Default::default()
        }
    }

    fn group_multivalue(&self) -> QString {
        QString::from(self.display_options.group_multivalue.to_string())
    }


    fn category_color(&self, category: QString) -> QString {
        if let Some(category) = self.display_options.categories.get(&category.to_string().to_lowercase()) {
            QString::from(category.color.clone().unwrap_or_else(|| String::from("#00000000")))
        }
        else {
            QString::from("#00000000")
        }
    }

    fn category_icon(&self, category: QString) -> QString {
        if let Some(category) = self.display_options.categories.get(&category.to_string().to_lowercase()) {
            QString::from(category.icon.clone().unwrap_or_default())
        }
        else {
            QString::from("")
        }
    }

    fn allow_collapsing_command(&self, command_id: QString) -> QString {
        if self.display_options.non_collapsible_commands.contains(&command_id.to_string().to_lowercase()) {
            QString::from("0")
        }
        else {
            QString::from("1")
        }
    }
}