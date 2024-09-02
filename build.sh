#!/usr/bin/env bash
set -eu
git submodule update --init --recursive
# qmake path can be overridden with this:
export QMAKE="/usr/lib/qt6/bin/qmake"
cargo build
