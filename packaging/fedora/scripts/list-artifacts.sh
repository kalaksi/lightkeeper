#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
    echo "usage: $0 <rpmbuild-topdir>" >&2
    exit 1
fi

TOPDIR="$1"

if [ -d "${TOPDIR}/RPMS" ]; then
    for file in "${TOPDIR}"/RPMS/*/*.rpm; do
        if [ -f "${file}" ]; then
            echo "${file}"
        fi
    done
fi

if [ -d "${TOPDIR}/SRPMS" ]; then
    for file in "${TOPDIR}"/SRPMS/*.src.rpm; do
        if [ -f "${file}" ]; then
            echo "${file}"
        fi
    done
fi
