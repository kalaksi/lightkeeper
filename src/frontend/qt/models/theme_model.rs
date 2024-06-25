extern crate qmetaobject;
use qmetaobject::*;
use std::str::FromStr;

use crate::{configuration, enums::Criticality};

// NOTE: See also qtquickcontrols2.conf for manually set color palette.

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct ThemeModel {
    base: qt_base_class!(trait QObject),
    disabledTextColor: qt_property!(QString; CONST),
    iconColor: qt_property!(QString; CONST),
    textColor: qt_property!(QString; CONST),
    textColorDark: qt_property!(QString; CONST),
    borderColor: qt_property!(QString; CONST),
    titleBarColor: qt_property!(QString; CONST),
    highlightColor: qt_property!(QString; CONST),
    highlightColorLight: qt_property!(QString; CONST),
    highlightColorBright: qt_property!(QString; CONST),
    backgroundColorDark: qt_property!(QString; CONST),
    backgroundColor: qt_property!(QString; CONST),
    backgroundColorLight: qt_property!(QString; CONST),
    categoryBackgroundColor: qt_property!(QString; CONST),
    categoryRefreshMask: qt_property!(QString; CONST),
    spacingLoose: qt_property!(i8; CONST),
    spacingNormal: qt_property!(i8; CONST),
    spacingTight: qt_property!(i8; CONST),
    marginScrollbar: qt_property!(i8; CONST),
    marginDialog: qt_property!(i8; CONST),
    marginDialogTop: qt_property!(i8; CONST),
    // Content will often overflow behind the dialog buttons (ugh...), reserve more space for them with this.
    marginDialogBottom: qt_property!(i8; CONST),
    animationDuration: qt_property!(i32; CONST),
    animationDurationFast: qt_property!(i32; CONST),
    groupboxMinWidth: qt_property!(i32; CONST),
    groupboxMaxWidth: qt_property!(i32; CONST),
    commonIndent: qt_property!(i32; CONST),

    // Display options
    hideInfoNotifications: qt_property!(bool; CONST),
    showStatusBar: qt_property!(bool; CONST),
    tooltipDelay: qt_property!(i32; CONST),

    categoryColor: qt_method!(fn(&self, category: QString) -> QString),
    categoryIcon: qt_method!(fn(&self, category: QString) -> QString),
    colorForCriticality: qt_method!(fn(&self, criticality: QString) -> QString),
    iconForCriticality: qt_method!(fn(&self, alert_level: QString) -> QString),
    opacity: qt_method!(fn(&self, is_enabled: bool) -> QString),
    getDisplayOptions: qt_method!(fn(&self) -> QVariant),
    allowCollapsingCommand: qt_method!(fn(&self, command_id: QString) -> QString),


    i_display_options: configuration::DisplayOptions,
}

#[allow(non_snake_case)]
impl ThemeModel {
    pub fn new(display_options: configuration::DisplayOptions) -> ThemeModel {
        // TODO: Utilize Kirigami.Theme and Kirigami.Units? Won't have everything to be sufficient, but
        // could be used to set values to this model. Or maybe use Kirigami's models and leave anything extra here?
        // OTOH, this model could be easier to use and more flexible.
        ThemeModel {
            disabledTextColor: QString::from("#50fcfcfc"),
            iconColor: QString::from("#a0a0a0"),
            textColor: QString::from("#fcfcfc"),
            textColorDark: QString::from("#a0a0a0"),
            borderColor: QString::from("#505050"),
            titleBarColor: QString::from("#404040"),
            highlightColor: QString::from("#242628"),
            highlightColorLight: QString::from("#30ffffff"),
            highlightColorBright: QString::from("#50ff2222"),
            backgroundColorDark: QString::from("#252525"),
            backgroundColor: QString::from("#2a2e32"),
            backgroundColorLight: QString::from("#303030"),
            categoryBackgroundColor: QString::from("#404040"),
            categoryRefreshMask: QString::from("#90404040"),
            spacingLoose: 12,
            spacingNormal: 8,
            spacingTight: 2,
            marginScrollbar: 16,
            marginDialog: 30,
            marginDialogTop: 45,
            marginDialogBottom: 80,
            animationDuration: 175,
            animationDurationFast: 80,
            groupboxMinWidth: 450,
            groupboxMaxWidth: 650,
            commonIndent: 16,
            tooltipDelay: 800,

            hideInfoNotifications: display_options.hide_info_notifications,
            showStatusBar: display_options.show_status_bar,
            i_display_options: display_options,
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

    fn categoryIcon(&self, category: QString) -> QString {
        if let Some(category) = self.i_display_options.categories.get(&category.to_string()) {
            QString::from(category.icon.clone().unwrap_or_default())
        }
        else {
            QString::from("")
        }
    }

    fn opacity(&self, is_enabled: bool) -> QString {
        if is_enabled {
            return QString::from("1.0");
        }
        QString::from("0.3")
    }

    // TODO: better place.
    fn allowCollapsingCommand(&self, command_id: QString) -> QString {
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

    fn getDisplayOptions(&self) -> QVariant {
        self.i_display_options.to_qvariant()
    }

    fn iconForCriticality(&self, criticality: QString) -> QString {
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
}