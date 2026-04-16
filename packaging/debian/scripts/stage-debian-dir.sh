#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ]; then
    echo "usage: $0 <unpacked-source-dir>" >&2
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../../.." && pwd)
SOURCE_DIR="$1"

mkdir -p "${SOURCE_DIR}"
cp -a "${REPO_ROOT}/packaging/debian/debian" "${SOURCE_DIR}/debian"
