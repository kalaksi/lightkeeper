#[allow(unused_imports)]
use qmetaobject::qrc;

// Enabled only for non-debug builds. Debug-builds use file paths directly to avoid recompilation.
#[cfg(not(debug_assertions))]
pub fn init_resources() {
    qrc!(resources_qml, 
        "src/frontend/qt/qml" as "qml" {
            "Main.qml",
            "DebugRectangle.qml",
            "DynamicObjectManager.qml",
            "HostStatus.qml",
            "HostTable.qml",
            "JsonTextFormat.qml",
            "LightkeeperTray.qml",
            "MainMenuBar.qml",
            "MonitorSummary.qml",
            "WaveAnimation.qml",
            "RowHighlight.qml",
            "SnackbarContainer.qml",
            "Snackbar.qml",
            "StatusBar.qml",
            "TableCell.qml",
            "WorkingSprite.qml",
            "Button/AutoRefreshButton.qml",
            "Button/CommandButton.qml",
            "Button/ImageButton.qml",
            "Button/RefreshButton.qml",
            "ChartsView/ChartsView.qml",
            "DetailsView/CategoryGroupBox.qml",
            "DetailsView/CommandButtonRow.qml",
            "DetailsView/CustomCommandGroupBox.qml",
            "DetailsView/GroupBoxLabel.qml",
            "DetailsView/Header.qml",
            "DetailsView/HostDetailsLogView.qml",
            "DetailsView/HostDetailsMainView.qml",
            "DetailsView/HostDetails.qml",
            "DetailsView/HostDetailsTerminalView.qml",
            "DetailsView/HostDetailsTextEditorView.qml",
            "DetailsView/HostDetailsTextView.qml",
            "DetailsView/HostGroupBox.qml",
            "DetailsView/LogList.qml",
            "DetailsView/PropertyTableCell.qml",
            "DetailsView/PropertyTable.qml",
            "Dialog/CertificateMonitorDialog.qml",
            "Dialog/CommandOutputDialog.qml",
            "Dialog/ConfigHelperDialog.qml",
            "Dialog/ConfirmationDialog.qml",
            "Dialog/DialogBackground.qml",
            "Dialog/GroupConfigurationDialog.qml",
            "Dialog/HostConfigurationDialog.qml",
            "Dialog/HotkeyHelp.qml",
            "Dialog/InputDialog.qml",
            "Dialog/LightkeeperDialog.qml",
            "Dialog/ModuleSettingsDialog.qml",
            "Dialog/PreferencesDialog.qml",
            "Dialog/TextDialog.qml",
            "js/Parse.js",
            "js/TextTransform.js",
            "js/Test.js",
            "js/Utils.js",
            "js/ValueUnit.js",
            "Misc/BorderRectangle.qml",
            "Misc/LKTabButton.qml",
            "Misc/CooldownTimer.qml",
            "Misc/SemiCircle.qml",
            "Misc/OverlayImage.qml",
            "Tests/Test.qml",
            "Text/AlertText.qml",
            "Text/BaseText.qml",
            "Text/BigText.qml",
            "Text/NormalText.qml",
            "Text/OptionalText.qml",
            "Text/PillText.qml",
            "Text/PixelatedText.qml",
            "Text/ScrollableNormalText.qml",
            "Text/ScrollableSmallerText.qml",
            "Text/SmallerText.qml",
            "Text/SmallText.qml",
        }
    );
    resources_qml();
}