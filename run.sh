#!/usr/bin/env bash
set -eu

# With different Qt theme:
# QT_QUICK_CONTROLS_STYLE=org.kde.desktop \
# QT_QUICK_CONTROLS_STYLE=Material \
# QT_STYLE_OVERRIDE=Breeze \
RUST_LOG=debug ./target/debug/lightkeeper
