extern crate qmetaobject;
use qmetaobject::*;
use std::str::FromStr;

use crate::{configuration, enums::Criticality};

// NOTE: See also qtquickcontrols2.conf for manually set color palette.

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct ThemeModel {
    base: qt_base_class!(trait QObject),

    // NOTE: New way of doing things. Use Qt properties and camelCase.
    disabledTextColor: qt_property!(QString; CONST),
    iconColor: qt_property!(QString; CONST),
    textColor: qt_property!(QString; CONST),
    textColorDark: qt_property!(QString; CONST),
    borderColor: qt_property!(QString; CONST),
    highlightColorBright: qt_property!(QString; CONST),
    backgroundColorDark: qt_property!(QString; CONST),
    backgroundColor: qt_property!(QString; CONST),
    backgroundColorLight: qt_property!(QString; CONST),
    tableBackgroundColor: qt_property!(QString; CONST),
    spacingLoose: qt_property!(i8; CONST),
    spacingNormal: qt_property!(i8; CONST),
    spacingTight: qt_property!(i8; CONST),
    marginScrollbar: qt_property!(i8; CONST),
    animationDuration: qt_property!(i32; CONST),
    groupboxMinWidth: qt_property!(i32; CONST),
    groupboxMaxWidth: qt_property!(i32; CONST),

    categoryColor: qt_method!(fn(&self, category: QString) -> QString),
    categoryIcon: qt_method!(fn(&self, category: QString) -> QString),
    colorForCriticality: qt_method!(fn(&self, criticality: QString) -> QString),

    // NOTE: Old methods, will be deprecated.
    category_background_color: qt_method!(fn(&self) -> QString),
    category_refresh_mask: qt_method!(fn(&self) -> QString),

    margin_dialog: qt_method!(fn(&self) -> i8),
    // Content will often overflow behind the dialog buttons (ugh...), reserve more space for them with this.
    margin_dialog_bottom: qt_method!(fn(&self) -> i8),
    spacing_loose: qt_method!(fn(&self) -> i8),
    spacing_normal: qt_method!(fn(&self) -> i8),
    spacing_tight: qt_method!(fn(&self) -> i8),
    common_indentation: qt_method!(fn(&self) -> i8),

    // TODO: deprecated
    background_color: qt_method!(fn(&self) -> QString),
    color_highlight: qt_method!(fn(&self) -> QString),
    color_highlight_light: qt_method!(fn(&self) -> QString),

    opacity: qt_method!(fn(&self, is_enabled: bool) -> QString),

    allow_collapsing_command: qt_method!(fn(&self, command_id: QString) -> QString),
    tooltip_delay: qt_method!(fn(&self) -> QVariant),
    animation_duration: qt_method!(fn(&self) -> QVariant),
    get_display_options: qt_method!(fn(&self) -> QVariant),
    icon_for_criticality: qt_method!(fn(&self, alert_level: QString) -> QString),
    hide_info_notifications: qt_method!(fn(&self) -> bool),
    notification_show_duration: qt_method!(fn(&self) -> i32),

    i_display_options: configuration::DisplayOptions,
}

#[allow(non_snake_case)]
impl ThemeModel {
    pub fn new(display_options: configuration::DisplayOptions) -> ThemeModel {
        // TODO: Utilize Kirigami.Theme and Kirigami.Units? Won't have everything to be sufficient, but
        // could be used to set values to this model. Or maybe use Kirigami's models and leave anything extra here?
        // OTOH, this model could be easier to use and more flexible.
        ThemeModel {
            i_display_options: display_options,
            disabledTextColor: QString::from("#3a4045"),
            iconColor: QString::from("#a0a0a0"),
            textColor: QString::from("#fcfcfc"),
            textColorDark: QString::from("#a0a0a0"),
            borderColor: QString::from("#505050"),
            highlightColorBright: QString::from("#50ff2222"),
            backgroundColorDark: QString::from("#252525"),
            backgroundColor: QString::from("#2a2e32"),
            backgroundColorLight: QString::from("#303030"),
            tableBackgroundColor: QString::from("#26292d"),
            spacingLoose: 12,
            spacingNormal: 8,
            spacingTight: 2,
            marginScrollbar: 16,
            animationDuration: 175,
            groupboxMinWidth: 450,
            groupboxMaxWidth: 650,
            ..Default::default()
        }
    }

    fn categoryColor(&self, category: QString) -> QString {
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

    fn categoryIcon(&self, category: QString) -> QString {
        if let Some(category) = self.i_display_options.categories.get(&category.to_string()) {
            QString::from(category.icon.clone().unwrap_or_default())
        }
        else {
            QString::from("")
        }
    }

    fn margin_dialog(&self) -> i8 {
        30
    }

    fn margin_dialog_bottom(&self) -> i8 {
        80
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

    fn background_color(&self) -> QString {
        QString::from("#2a2e32")
    }

    fn color_highlight(&self) -> QString {
        QString::from("#242628")
    }

    fn color_highlight_light(&self) -> QString {
        QString::from("#30ffffff")
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

    fn colorForCriticality(&self, criticality: QString) -> QString {
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
        5000
    }
}