#!/usr/bin/env bash
set -eu
# qmake path can be overridden with this (for cargo and qmetaobject-rs too):
export QMAKE="/usr/lib/qt6/bin/qmake"

if [ ! -e third_party/qmltermwidget/QMLTermWidget/libqmltermwidget.so ]; then
    git submodule update --init --recursive
    pushd third_party/qmltermwidget
    $QMAKE && make
    popd
fi

cargo build
