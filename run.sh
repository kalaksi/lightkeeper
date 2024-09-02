#!/usr/bin/env bash
set -eu

# With different Qt theme:
# QT_QUICK_CONTROLS_STYLE=org.kde.desktop \
# QT_QUICK_CONTROLS_STYLE=Material \
# QT_STYLE_OVERRIDE=Breeze \
QML_IMPORT_PATH="/home/user/git/qmltermwidget/QMLTermWidget" \
LD_LIBRARY_PATH="/home/user/git/qmltermwidget/QMLTermWidget" \
RUST_LOG=debug ./target/debug/lightkeeper
