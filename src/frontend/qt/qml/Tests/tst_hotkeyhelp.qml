import QtQuick 2.15
import QtQuick.Controls 2.15
import QtTest 1.2

import "../Dialog"


TestCase {
    id: root
    name: "StartupTest"

    Component {
        id: appComponent

        HotkeyHelp {
        }
    }

    property var appRoot: null

    function initTestCase() {
        appRoot = appComponent.createObject(root)
        verify(appRoot !== null, "Application should be created successfully")
    }

    function cleanupTestCase() {
        appRoot.destroy()
    }

    function test_navigation() {
        compare("no", "Test Input", "Text field should contain 'Test Input'")
    }
}