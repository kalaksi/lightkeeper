use qmetaobject::qrc;

pub fn init_resources() {
    qrc!(resources_main, 
        "src/frontend/qt" as "main" {
            "fonts/Pixeloid/PixeloidSans-nR3g1.ttf" as "fonts/pixeloid",
            "fonts/PressStart2P/PressStart2P-vaV7.ttf" as "fonts/pressstart2p",

            "images/fontawesome/circle-exclamation.svg" as "images/criticality/critical",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/error",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/warning",
            "images/fontawesome/circle-check.svg" as "images/criticality/normal",
            "images/breeze/light/alarm-symbolic.svg" as "images/criticality/nodata",

            "images/fontawesome/circle-arrow-up.svg" as "images/status/up",
            "images/fontawesome/circle-arrow-down.svg" as "images/status/down",
            "images/breeze/dark/alarm-symbolic.svg" as "images/status/unknown",

            "images/fontawesome/circle-exclamation.svg" as "images/alert/error",
            "images/fontawesome/triangle-exclamation.svg" as "images/alert/warning",

            "images/breeze/dark/view-refresh.svg" as "images/button/refresh",
            "images/breeze/dark/system-search.svg" as "images/button/search",
            "images/breeze/dark/find-location.svg" as "images/button/search-document",
            "images/breeze/dark/document-preview.svg" as "images/button/view-document",
            // "images/breeze/dark/utilities-terminal.svg" as "images/button/terminal",
            "images/fontawesome/terminal.svg" as "images/button/terminal",

            "images/breeze/dark/process-working.svg" as "images/animations/working",
        }
    );
    resources_main();

    qrc!(resources_theme_light, 
        "src/frontend/qt" as "main" {
            "fonts/Pixeloid/PixeloidSans-nR3g1.ttf" as "fonts/pixeloid",
            "fonts/PressStart2P/PressStart2P-vaV7.ttf" as "fonts/pressstart2p",

            "images/fontawesome/circle-exclamation.svg" as "images/criticality/critical",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/error",
            "images/fontawesome/circle-exclamation.svg" as "images/criticality/warning",
            "images/fontawesome/circle-check.svg" as "images/criticality/normal",
            "images/breeze/light/alarm-symbolic.svg" as "images/criticality/nodata",

            "images/fontawesome/circle-arrow-up.svg" as "images/status/up",
            "images/fontawesome/circle-arrow-down.svg" as "images/status/down",
            "images/breeze/light/alarm-symbolic.svg" as "images/status/unknown",

            "images/fontawesome/circle-exclamation.svg" as "images/alert/error",
            "images/fontawesome/triangle-exclamation.svg" as "images/alert/warning",

            "images/breeze/light/view-refresh.svg" as "images/button/refresh",
            "images/breeze/light/system-search.svg" as "images/button/search",
            "images/breeze/light/find-location.svg" as "images/button/search-document",
            "images/breeze/light/utilities-terminal.svg" as "images/button/terminal",

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