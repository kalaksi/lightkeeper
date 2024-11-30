#!/usr/bin/env bash
set -eu
# qmake path can be overridden with this (for cargo and qmetaobject-rs too):
export QMAKE="/usr/lib/qt6/bin/qmake"

if [ ! -e third_party/qmltermwidget ] || \
   [ ! -e third_party/ChartJs2QML ]; then

    git submodule update --init --recursive
    # git submodule update --init --recursive --remote
fi

if [ ! -e third_party/qmltermwidget/QMLTermWidget/libqmltermwidget.so ]; then
    pushd third_party/qmltermwidget
    $QMAKE && make
    popd
fi

cargo build
