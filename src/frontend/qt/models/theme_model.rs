extern crate qmetaobject;
use qmetaobject::*;

use crate::configuration;


#[derive(QObject, Default)]
pub struct ThemeModel {
    base: qt_base_class!(trait QObject),
    group_multivalue: qt_method!(fn(&self) -> QString),
    category_color: qt_method!(fn(&self, category: QString) -> QString),
    category_icon: qt_method!(fn(&self, category: QString) -> QString),
    category_background_color: qt_method!(fn(&self) -> QString),
    allow_collapsing_command: qt_method!(fn(&self, command_id: QString) -> QString),
    tooltip_delay: qt_method!(fn(&self) -> QVariant),
    pill_color_for_criticality: qt_method!(fn(&self, criticality: QString) -> QString),

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
        if let Some(category) = self.display_options.categories.get(&category.to_string()) {
            QString::from(category.color.clone().unwrap_or_else(|| String::from("#505050")))
        }
        else {
            QString::from("#505050")
        }
    }

    fn category_background_color(&self) -> QString {
        QString::from(String::from("#404040"))
    }

    fn category_icon(&self, category: QString) -> QString {
        if let Some(category) = self.display_options.categories.get(&category.to_string()) {
            QString::from(category.icon.clone().unwrap_or_default())
        }
        else {
            QString::from("")
        }
    }

    fn allow_collapsing_command(&self, command_id: QString) -> QString {
        if self.display_options.non_collapsible_commands.contains(&command_id.to_string()) {
            QString::from("0")
        }
        else {
            QString::from("1")
        }
    }

    fn tooltip_delay(&self) -> QVariant {
        QVariant::from(800)
    }

    fn pill_color_for_criticality(&self, criticality: QString) -> QString {
        // TODO: Deserialize from serde and user enum instead.
        match criticality.to_string().as_str() {
            "Critical" => QString::from("#60ff3300"),
            "Error" => QString::from("#60ff3300"),
            "Warning" => QString::from("#60ffcc00"),
            "Normal" => QString::from("#6033cc33"),
            "Info" => QString::from("#60ffffff"),
            _ => QString::from("#60ffffff"),
        }
    }
}