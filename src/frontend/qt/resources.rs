use qmetaobject::qrc;

pub fn init_resources() {
    qrc!(resources_main, 
        "src/frontend/qt" as "main" {

            // Fonts
            "fonts/Pixeloid/PixeloidSans-nR3g1.ttf" as "fonts/pixeloid",
            "fonts/PressStart2P/PressStart2P-vaV7.ttf" as "fonts/pressstart2p",

            // Status or levels
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/critical",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/error",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/warning",
            "images/fontawesome/circle-check.svg" as "images/criticality/normal",
            "images/breeze/light/alarm-symbolic.svg" as "images/criticality/nodata",

            "images/fontawesome/circle-arrow-up.svg" as "images/status/up",
            "images/fontawesome/circle-arrow-down.svg" as "images/status/down",
            "images/breeze/dark/alarm-symbolic.svg" as "images/status/pending",
            "images/breeze/dark/alarm-symbolic.svg" as "images/status/unknown",

            "images/breeze/dark/data-information.svg" as "images/alert/info",
            "images/breeze/dark/data-warning.svg" as "images/alert/warning",
            "images/breeze/dark/data-error.svg" as "images/alert/error",

            // Buttons
            "images/breeze/dark/view-refresh.svg" as "images/button/refresh",
            "images/breeze/dark/system-search.svg" as "images/button/search",
            "images/breeze/dark/find-location.svg" as "images/button/search-document",
            "images/breeze/dark/document-preview.svg" as "images/button/view-document",
            "images/breeze/dark/akonadiconsole.svg" as "images/button/terminal",
            "images/breeze/dark/delete.svg" as "images/button/delete",
            "images/breeze/dark/edit-clear-all.svg" as "images/button/clear",
            "images/breeze/dark/system-shutdown.svg" as "images/button/shutdown",
            "images/breeze/dark/arrow-up.svg" as "images/button/search-up",
            "images/breeze/dark/arrow-down.svg" as "images/button/search-down",
            "images/breeze/dark/story-editor.svg" as "images/button/story-editor",
            "images/breeze/dark/download.svg" as "images/button/download",
            "images/breeze/dark/overflow-menu.svg" as "images/button/overflow-menu",
            "images/breeze/dark/dialog-ok.svg" as "images/button/ok",
            "images/breeze/dark/dialog-cancel.svg" as "images/button/cancel",
            "images/breeze/dark/tag.svg" as "images/button/tag",
            "images/breeze/dark/edit-copy.svg" as "images/button/copy",
            "images/breeze/dark/resizecol.svg" as "images/button/resize-column",
            "images/breeze/dark/gnumeric-column-size.svg" as "images/button/resize-column-2",
            "images/breeze/dark/list-add.svg" as "images/button/add",
            "images/breeze/dark/list-remove.svg" as "images/button/remove",
            "images/breeze/dark/configure.svg" as "images/button/configure",
            "images/breeze/dark/entry-edit.svg" as "images/button/entry-edit",
            "images/breeze/dark/group-new.svg" as "images/button/group-new",
            "images/breeze/dark/document-open.svg" as "images/button/document-open",
            "images/breeze/dark/document-open-folder.svg" as "images/button/document-open-folder",
            "images/breeze/dark/document-save.svg" as "images/button/document-save",
            "images/breeze/dark/run-build.svg" as "images/button/build",
            "images/breeze/dark/run-build-file.svg" as "images/button/build-file",
            "images/breeze/dark/help-keyboard-shortcuts.svg" as "images/button/keyboard-shortcuts",
            "images/breeze/dark/edit-undo.svg" as "images/button/undo",
            "images/breeze/dark/view-certificate.svg" as "images/button/certificates",
            "images/breeze/dark/arrow-down.svg" as "images/button/dropdown-menu",
            "images/breeze/dark/edit-clear.svg" as "images/button/edit-clear",
            "images/breeze/dark/project-open.svg" as "images/button/project-open",
            // "images/breeze/dark/office-chart-bar-stacked.svg" as "images/button/charts",
            "images/breeze/dark/office-chart-area-stacked.svg" as "images/button/charts",

            // Host icons
            "images/breeze/light/preferences-system-linux.svg" as "images/host/linux",

            // Tray icon
            "images/lightkeeper-tray-icon.png" as "images/tray-icon",

            // Category icons
            "images/fontawesome/docker.svg" as "images/docker",
            "images/nixos.svg" as "images/nixos",
            "images/breeze/dark/drive-harddisk.svg" as "images/harddisk",

            "images/breeze/dark/media-playback-start.svg" as "images/button/start",
            "images/breeze/dark/media-playback-stop.svg" as "images/button/stop",
            "images/breeze/dark/update-none.svg" as "images/button/update",
            "images/breeze/dark/update-low.svg" as "images/button/update-low",
            "images/breeze/dark/update-high.svg" as "images/button/update-high",

            // Control buttons
            "images/fontawesome/xmark.svg" as "images/button/close",
            "images/breeze/dark/tab-close.svg" as "images/button/tab-close",
            "images/breeze/dark/arrow-up.svg" as "images/button/maximize",
            "images/breeze/dark/arrow-down.svg" as "images/button/minimize",
            "images/breeze/dark/window-new.svg" as "images/button/window-new",

            // Animations
            "images/breeze/dark/process-working.svg" as "images/animations/working",
        }
    );
    resources_main();

    qrc!(resources_theme_light, 
        "src/frontend/qt" as "main" {
            // Fonts
            "fonts/Pixeloid/PixeloidSans-nR3g1.ttf" as "fonts/pixeloid",
            "fonts/PressStart2P/PressStart2P-vaV7.ttf" as "fonts/pressstart2p",

            // Status or levels
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/critical",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/error",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/warning",
            "images/fontawesome/circle-check.svg" as "images/criticality/normal",
            "images/breeze/light/alarm-symbolic.svg" as "images/criticality/nodata",

            "images/fontawesome/circle-arrow-up.svg" as "images/status/up",
            "images/fontawesome/circle-arrow-down.svg" as "images/status/down",
            "images/breeze/light/alarm-symbolic.svg" as "images/status/pending",
            "images/breeze/light/alarm-symbolic.svg" as "images/status/unknown",

            "images/breeze/light/data-information.svg" as "images/alert/info",
            "images/breeze/light/data-warning.svg" as "images/alert/warning",
            "images/breeze/light/data-error.svg" as "images/alert/error",

            // Buttons
            "images/breeze/light/view-refresh.svg" as "images/button/refresh",
            "images/breeze/light/system-search.svg" as "images/button/search",
            "images/breeze/light/find-location.svg" as "images/button/search-document",
            "images/breeze/light/akonadiconsole.svg" as "images/button/terminal",
            "images/breeze/light/delete.svg" as "images/button/delete",
            "images/breeze/light/edit-clear-all.svg" as "images/button/clear",
            "images/breeze/light/system-shutdown.svg" as "images/button/shutdown",
            "images/breeze/light/arrow-up.svg" as "images/button/search-up",
            "images/breeze/light/arrow-down.svg" as "images/button/search-down",
            "images/breeze/light/story-editor.svg" as "images/button/story-editor",
            "images/breeze/light/download.svg" as "images/button/download",
            "images/breeze/light/overflow-menu.svg" as "images/button/overflow-menu",
            "images/breeze/light/dialog-ok.svg" as "images/button/ok",
            "images/breeze/light/dialog-cancel.svg" as "images/button/cancel",
            "images/breeze/light/tag.svg" as "images/light/tag",
            "images/breeze/light/edit-copy.svg" as "images/light/copy",
            "images/breeze/light/resizecol.svg" as "images/button/resize-column",
            "images/breeze/light/gnumeric-column-size.svg" as "images/button/resize-column-2",
            "images/breeze/light/list-add.svg" as "images/button/add",
            "images/breeze/light/list-remove.svg" as "images/button/remove",
            "images/breeze/light/configure.svg" as "images/button/configure",
            "images/breeze/light/entry-edit.svg" as "images/button/entry-edit",
            "images/breeze/light/group-new.svg" as "images/button/group-new",
            "images/breeze/light/document-open.svg" as "images/button/document-open",
            "images/breeze/light/document-open-folder.svg" as "images/button/document-open-folder",
            "images/breeze/light/document-save.svg" as "images/button/document-save",
            "images/breeze/light/run-build.svg" as "images/button/build",
            "images/breeze/light/run-build-file.svg" as "images/button/build-file",
            "images/breeze/light/help-keyboard-shortcuts.svg" as "images/button/keyboard-shortcuts",
            "images/breeze/light/edit-undo.svg" as "images/button/undo",
            "images/breeze/light/view-certificate.svg" as "images/button/certificates",
            "images/breeze/light/arrow-down.svg" as "images/button/dropdown-menu",
            "images/breeze/light/edit-clear.svg" as "images/button/edit-clear",
            "images/breeze/light/project-open.svg" as "images/button/project-open",
            // "images/breeze/light/office-chart-bar-stacked.svg" as "images/button/charts",
            "images/breeze/light/office-chart-area-stacked.svg" as "images/button/charts",

            // Host icons
            "images/breeze/light/preferences-system-linux.svg" as "images/host/linux",

            // Tray icon
            "images/lightkeeper-tray-icon.png" as "images/tray-icon",

            // Category icons
            "images/fontawesome/docker.svg" as "images/docker",
            "images/nixos.svg" as "images/nixos",
            "images/breeze/light/drive-harddisk.svg" as "images/harddisk",

            "images/breeze/light/media-playback-start.svg" as "images/button/start",
            "images/breeze/light/media-playback-stop.svg" as "images/button/stop",
            "images/breeze/light/update-none.svg" as "images/button/update",
            "images/breeze/light/update-low.svg" as "images/button/update-low",
            "images/breeze/light/update-high.svg" as "images/button/update-high",

            // Control buttons
            "images/fontawesome/xmark.svg" as "images/button/close",
            "images/breeze/light/tab-close.svg" as "images/button/tab-close",
            "images/breeze/light/arrow-up.svg" as "images/button/maximize",
            "images/breeze/light/arrow-down.svg" as "images/button/minimize",
            "images/breeze/light/window-new.svg" as "images/button/window-new",

            // Animations
            "images/breeze/light/process-working.svg" as "images/animations/working",
        }
    );
    resources_theme_light();
}