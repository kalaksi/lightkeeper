/*
 * SPDX-FileCopyrightText: Copyright (C) 2025 kalaksi@users.noreply.github.com
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtTest 1.2

import "../"
import "../js/Test.js" as TestUtils


TestCase {
    id: root
    name: "StartupTest"

    function run(app) {
        testCheckInitialStatus(app)
    }
    // Component {
    //     id: app

    //     Main {
    //     }
    // }

    // property var appRoot: null

    // function initTestCase() {
    //     appRoot = appComponent.createObject(root)
    //     verify(appRoot !== null, "Application should be created successfully")
    // }

    // function cleanupTestCase() {
    //     appRoot.destroy()
    // }

    // function test_navigation() {
    //     compare("no", "Test Input", "Text field should contain 'Test Input'")
    // }
}