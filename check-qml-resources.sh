#!/usr/bin/env sh

qml_files=$(find src/frontend/qt/qml \( -name '*.qml' -or -name '*.js' \) | sed 's/^src\/frontend\/qt\/qml\///')
for f in $qml_files; do
    if ! fgrep -q "\"$f\"" "src/frontend/qt/resources_qml.rs"; then
        echo "File $f not found in QML resource file"
        exit 1
    fi
done
