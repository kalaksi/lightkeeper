#!/usr/bin/env bash
set -eu

# With different Qt theme:
# export QT_QUICK_CONTROLS_STYLE=org.kde.desktop
# export QT_QUICK_CONTROLS_STYLE=Material
# export QT_STYLE_OVERRIDE=Breeze

# For testing QML modules:
# export QML2_IMPORT_PATH="./third_party/qmltermwidget"
# For debugging imports:
# export QML_IMPORT_TRACE=1

RUST_LOG=debug ./target/debug/lightkeeper
