extern crate qmetaobject;
use qmetaobject::*;
use std::str::FromStr;

use crate::{configuration, enums::Criticality};


#[derive(QObject, Default)]
pub struct ThemeModel {
    base: qt_base_class!(trait QObject),

    category_color: qt_method!(fn(&self, category: QString) -> QString),
    category_background_color: qt_method!(fn(&self) -> QString),
    category_refresh_mask: qt_method!(fn(&self) -> QString),
    category_icon: qt_method!(fn(&self, category: QString) -> QString),
    groupbox_padding: qt_method!(fn(&self) -> i8),
    allow_collapsing_command: qt_method!(fn(&self, command_id: QString) -> QString),
    tooltip_delay: qt_method!(fn(&self) -> QVariant),
    pill_color_for_criticality: qt_method!(fn(&self, criticality: QString) -> QString),
    get_display_options: qt_method!(fn(&self) -> QVariant),

    i_display_options: configuration::DisplayOptions,
}

impl ThemeModel {
    pub fn new(display_options: configuration::DisplayOptions) -> ThemeModel {
        ThemeModel {
            i_display_options: display_options,
            ..Default::default()
        }
    }

    fn category_color(&self, category: QString) -> QString {
        if let Some(category) = self.i_display_options.categories.get(&category.to_string()) {
            QString::from(category.color.clone().unwrap_or_else(|| String::from("#505050")))
        }
        else {
            QString::from("#505050")
        }
    }

    fn category_background_color(&self) -> QString {
        QString::from(String::from("#404040"))
    }

    fn category_refresh_mask(&self) -> QString {
        QString::from(String::from("#90404040"))
    }

    fn category_icon(&self, category: QString) -> QString {
        if let Some(category) = self.i_display_options.categories.get(&category.to_string()) {
            QString::from(category.icon.clone().unwrap_or_default())
        }
        else {
            QString::from("")
        }
    }

    fn groupbox_padding(&self) -> i8 {
        2
    }

    fn allow_collapsing_command(&self, command_id: QString) -> QString {
        // TODO: take category into consideration instead of accepting matching id from any of them.
        let allows_collapsing = self.i_display_options.categories.values().any(|category| {
            match &category.collapsible_commands {
                Some(collapsible_commands) => collapsible_commands.contains(&command_id.to_string()),
                None => false,
            }
        });

        if allows_collapsing {
            QString::from("1")
        }
        else {
            QString::from("0")
        }
    }

    fn tooltip_delay(&self) -> QVariant {
        QVariant::from(800)
    }

    fn pill_color_for_criticality(&self, criticality: QString) -> QString {
        match Criticality::from_str(&criticality.to_string()).unwrap() {
            Criticality::Critical => QString::from("#60ff3300"),
            Criticality::Error => QString::from("#60ff3300"),
            Criticality::Warning => QString::from("#60ffcc00"),
            Criticality::Normal => QString::from("#6033cc33"),
            Criticality::Info => QString::from("#60ffffff"),
            _ => QString::from("#60ffffff"),
        }
    }

    fn get_display_options(&self) -> QVariant {
        self.i_display_options.to_qvariant()
    }
}