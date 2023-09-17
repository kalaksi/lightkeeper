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
    groupbox_min_width: qt_method!(fn(&self) -> i32),
    groupbox_max_width: qt_method!(fn(&self) -> i32),

    margin_dialog: qt_method!(fn(&self) -> i8),
    // Content will often overflow behind the dialog buttons (ugh...), reserve more space for them with this.
    margin_dialog_bottom: qt_method!(fn(&self) -> i8),
    margin_scrollbar: qt_method!(fn(&self) -> i8),
    spacing_loose: qt_method!(fn(&self) -> i8),
    spacing_normal: qt_method!(fn(&self) -> i8),
    spacing_tight: qt_method!(fn(&self) -> i8),
    common_indentation: qt_method!(fn(&self) -> i8),

    color_red: qt_method!(fn(&self) -> QString),
    color_green: qt_method!(fn(&self) -> QString),
    color_yellow: qt_method!(fn(&self) -> QString),
    // TODO: deprecated
    background_color: qt_method!(fn(&self) -> QString),
    color_background: qt_method!(fn(&self) -> QString),
    color_background_2: qt_method!(fn(&self) -> QString),
    color_table_background: qt_method!(fn(&self) -> QString),
    color_text: qt_method!(fn(&self) -> QString),
    color_dark_text: qt_method!(fn(&self) -> QString),
    color_highlight: qt_method!(fn(&self) -> QString),
    color_highlight_light: qt_method!(fn(&self) -> QString),
    color_highlight_bright: qt_method!(fn(&self) -> QString),

    border_radius: qt_method!(fn(&self) -> i8),

    opacity: qt_method!(fn(&self, is_enabled: bool) -> QString),

    allow_collapsing_command: qt_method!(fn(&self, command_id: QString) -> QString),
    tooltip_delay: qt_method!(fn(&self) -> QVariant),
    animation_duration: qt_method!(fn(&self) -> QVariant),
    pill_color_for_criticality: qt_method!(fn(&self, criticality: QString) -> QString),
    get_display_options: qt_method!(fn(&self) -> QVariant),
    icon_for_criticality: qt_method!(fn(&self, alert_level: QString) -> QString),
    hide_info_notifications: qt_method!(fn(&self) -> bool),
    notification_show_duration: qt_method!(fn(&self) -> i32),

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

    fn groupbox_min_width(&self) -> i32 {
        450
    }

    fn groupbox_max_width(&self) -> i32 {
        650
    }

    fn margin_dialog(&self) -> i8 {
        30
    }

    fn margin_dialog_bottom(&self) -> i8 {
        80
    }

    fn margin_scrollbar(&self) -> i8 {
        16
    }

    fn spacing_loose(&self) -> i8 {
        12
    }

    fn spacing_normal(&self) -> i8 {
        8
    }

    fn spacing_tight(&self) -> i8 {
        2
    }

    fn common_indentation(&self) -> i8 {
        16
    }

    fn color_red(&self) -> QString {
        QString::from("#d05252")
    }

    fn color_green(&self) -> QString {
        QString::from("#4caa4f")
    }

    fn color_yellow(&self) -> QString {
        QString::from("#d3cc0a")
    }

    fn background_color(&self) -> QString {
        QString::from("#2a2e32")
    }

    fn color_background(&self) -> QString {
        QString::from("#2a2e32")
    }
    
    fn color_background_2(&self) -> QString {
        QString::from(String::from("#404040"))
    }

    fn color_table_background(&self) -> QString {
        QString::from("#26292d")
    }

    fn color_text(&self) -> QString {
        QString::from("#ffffff")
    }

    fn color_dark_text(&self) -> QString {
        QString::from("#a0a0a0")
    }

    fn color_highlight(&self) -> QString {
        QString::from("#242628")
    }

    fn color_highlight_light(&self) -> QString {
        QString::from("#30ffffff")
    }

    fn color_highlight_bright(&self) -> QString {
        QString::from("#50ff2222")
    }

    fn border_radius(&self) -> i8 {
        4
    }

    fn opacity(&self, is_enabled: bool) -> QString {
        if is_enabled {
            return QString::from("1.0");
        }
        QString::from("0.3")
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

    fn animation_duration(&self) -> QVariant {
        QVariant::from(175)
    }

    fn pill_color_for_criticality(&self, criticality: QString) -> QString {
        let criticality = criticality.to_string();

        if criticality.is_empty() {
            return QString::from("#60ffffff");
        }

        match Criticality::from_str(&criticality).unwrap() {
            Criticality::Ignore => QString::from("#60ffffff"),
            Criticality::Normal => QString::from("#6033cc33"),
            Criticality::Info => QString::from("#60ffffff"),
            Criticality::NoData => QString::from("#60ffffff"),
            Criticality::Warning => QString::from("#60ffcc00"),
            Criticality::Error => QString::from("#60ff3300"),
            Criticality::Critical => QString::from("#60ff3300"),
        }
    }

    fn get_display_options(&self) -> QVariant {
        self.i_display_options.to_qvariant()
    }

    fn icon_for_criticality(&self, criticality: QString) -> QString {
        let criticality = criticality.to_string();

        if criticality.is_empty() {
            return QString::from("qrc:/main/images/alert/info");
        }

        match Criticality::from_str(&criticality).unwrap() {
            Criticality::Ignore => QString::from("qrc:/main/images/alert/info"),
            Criticality::Normal => QString::from("qrc:/main/images/alert/info"),
            Criticality::Info => QString::from("qrc:/main/images/alert/info"),
            Criticality::NoData => QString::from("qrc:/main/images/alert/warning"),
            Criticality::Warning => QString::from("qrc:/main/images/alert/warning"),
            Criticality::Error => QString::from("qrc:/main/images/alert/error"),
            Criticality::Critical => QString::from("qrc:/main/images/alert/error"),
        }
    }

    fn hide_info_notifications(&self) -> bool {
        self.i_display_options.hide_info_notifications
    }

    fn notification_show_duration(&self) -> i32 {
        4000
    }
}