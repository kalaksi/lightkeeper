#!/usr/bin/env bash
set -eu
# qmake path can be overridden with this (for cargo and qmetaobject-rs too):
# export QMAKE="/usr/lib/qt6/bin/qmake"

if [ ! -e "third_party/qmltermwidget" ] || \
   [ ! -e "third_party/ChartJs2QML" ] || \
   [ ! -e "third_party/qml-lighthouse-ace-editor" ] \
   ; then

    git submodule update --init --recursive --remote
fi

if [ ! -e third_party/qmltermwidget/QMLTermWidget/libqmltermwidget.so ]; then
    pushd third_party/qmltermwidget
    qmake6 && make -j 4
    popd
fi

if [ ! -z "$(git status -s)" ]; then
    # Expand use later. Currently, rustfmt in many cases makes readability worse by splitting or joining lines badly.
    rustfmt +nightly src/utils.rs \
        src/metrics.rs \
        src/enums.rs \
        src/error.rs \
        src/host.rs \
        src/configuration.rs \
        src/module/module_factory.rs \
        src/module/metadata.rs \
        src/module/platform_info.rs \
        src/frontend/display_options.rs \
        src/frontend/hot_reload.rs \
        src/frontend/qt/qml_frontend.rs \
        src/frontend/qt/resources.rs \
        src/frontend/qt/resources_qml.rs \
        src/file_handler.rs
fi

cargo build
