#!/usr/bin/env bash
set -euo pipefail

BUILDDIR="${1:-}"
if [ -z "${BUILDDIR}" ]; then
    echo "usage: $0 <debian-build-dir>" >&2
    exit 1
fi

for ext in deb changes buildinfo dsc tar.xz; do
    for file in "${BUILDDIR}"/*.${ext}; do
        if [ -f "${file}" ]; then
            echo "${file}"
        fi
    done
done

for file in "${BUILDDIR}"/*.orig.tar.gz; do
    if [ -f "${file}" ]; then
        echo "${file}"
    fi
done
