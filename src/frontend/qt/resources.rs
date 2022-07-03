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
            "images/fontawesome/circle-question.svg" as "images/criticality/unknown",

            "images/fontawesome/circle-arrow-up.svg" as "images/status/up",
            "images/fontawesome/circle-arrow-down.svg" as "images/status/down",
            "images/fontawesome/circle-question.svg" as "images/status/unknown",

            "images/breeze/dark/view-refresh.svg" as "images/button/refresh",
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
            "images/fontawesome/circle-question.svg" as "images/criticality/unknown",

            "images/fontawesome/circle-arrow-up.svg" as "images/status/up",
            "images/fontawesome/circle-arrow-down.svg" as "images/status/down",
            "images/fontawesome/circle-question.svg" as "images/status/unknown",

            "images/breeze/light/view-refresh.svg" as "images/button/refresh",
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