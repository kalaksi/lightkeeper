#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ]; then
    echo "usage: $0 <version> <tarball-dir> <build-dir>" >&2
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../../.." && pwd)

VERSION="$1"
OUTDIR="$(mkdir -p "$2" && cd "$2" && pwd)"
BUILDDIR="$(mkdir -p "$3" && cd "$3" && pwd)"

TARBALL="$("${REPO_ROOT}/packaging/scripts/source-tarball.sh" "${VERSION}" "${OUTDIR}")"

tar -xzf "${TARBALL}" -C "${BUILDDIR}"
"${SCRIPT_DIR}/stage-debian-dir.sh" "${BUILDDIR}/lightkeeper-${VERSION}"

cp "${TARBALL}" "${BUILDDIR}/lightkeeper_${VERSION}.orig.tar.gz"

pushd "${BUILDDIR}/lightkeeper-${VERSION}" >/dev/null
dpkg-buildpackage -S -sa -us -uc -d
popd >/dev/null

echo "DEBIAN_BUILD_DIR=${BUILDDIR}"
