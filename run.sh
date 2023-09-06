#!/usr/bin/env bash
set -eu

# With different Qt theme:
# QT_QUICK_CONTROLS_STYLE=Universal \
RUST_LOG=debug ./target/debug/lightkeeper
