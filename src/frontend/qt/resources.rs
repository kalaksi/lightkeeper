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
            // "images/breeze/dark/utilities-terminal.svg" as "images/button/terminal",
            "images/fontawesome/terminal.svg" as "images/button/terminal",
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

            // Host icons
            "images/breeze/light/preferences-system-linux.svg" as "images/host/linux",

            // Category icons
            "images/fontawesome/docker.svg" as "images/docker",

            "images/breeze/dark/media-playback-start.svg" as "images/button/start",
            "images/breeze/dark/media-playback-stop.svg" as "images/button/stop",
            "images/breeze/dark/update-none.svg" as "images/button/update",
            "images/breeze/dark/update-low.svg" as "images/button/update-low",
            "images/breeze/dark/update-high.svg" as "images/button/update-high",

            "images/fontawesome/xmark.svg" as "images/button/close",
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
            "images/breeze/light/utilities-terminal.svg" as "images/button/terminal",
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

            // Host icons
            "images/breeze/light/preferences-system-linux.svg" as "images/host/linux",

            // Category icons
            "images/fontawesome/docker.svg" as "images/docker",

            "images/breeze/light/media-playback-start.svg" as "images/button/start",
            "images/breeze/light/media-playback-stop.svg" as "images/button/stop",
            "images/breeze/light/update-none.svg" as "images/button/update",
            "images/breeze/light/update-low.svg" as "images/button/update-low",
            "images/breeze/light/update-high.svg" as "images/button/update-high",

            "images/fontawesome/xmark.svg" as "images/button/close",
            "images/breeze/light/arrow-up.svg" as "images/button/maximize",
            "images/breeze/light/arrow-down.svg" as "images/button/minimize",
            "images/breeze/light/window-new.svg" as "images/button/window-new",

            // Animations
            "images/breeze/light/process-working.svg" as "images/animations/working",
        }
    );
    resources_theme_light();

    // qrc!(resources_theme1, 
    //     "src/frontend/qt" as "main" {
    //         "fonts/Pixeloid/PixeloidSans-nR3g1.ttf" as "fonts/pixeloid",
    //         "fonts/PressStart2P/PressStart2P-vaV7.ttf" as "fonts/pressstart2p",

    //         "images/fontawesome/circle-exclamation.svg" as "images/criticality/critical",
    //         "images/fontawesome/circle-exclamation.svg" as "images/criticality/error",
    //         "images/fontawesome/circle-exclamation.svg" as "images/criticality/warning",
    //         "images/fontawesome/circle-check.svg" as "images/criticality/normal",
    //         "images/fontawesome/circle-question.svg" as "images/criticality/unknown",

    //         "images/fontawesome/check.svg" as "images/status/up",
    //         "images/fontawesome/exclamation.svg" as "images/status/down",
    //         "images/fontawesome/question.svg" as "images/status/unknown",

    //         "images/fontawesome/chevron-down.svg" as "images/button/chevron-down"
    //     }
    // );
    // resources_theme1();

}