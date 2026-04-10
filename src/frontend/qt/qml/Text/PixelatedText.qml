/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick

import Lightkeeper 1.0

Text {
    color: Theme.textColorDark
    font.pixelSize: 8
    font.family: fontLabel.name
    antialiasing: false
    font.letterSpacing: 1

    FontLoader {
        id: fontLabel
        source: "qrc:/main/fonts/pixeloid"
    }
}