#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 3 ]; then
    echo "usage: $0 <version> <rpmbuild-topdir> <tarball-dir>" >&2
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../../.." && pwd)

VERSION="$1"
TOPDIR="$(mkdir -p "$2" && cd "$2" && pwd)"
OUTDIR="$(mkdir -p "$3" && cd "$3" && pwd)"

mkdir -p "${TOPDIR}/"{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

TARBALL="$("${REPO_ROOT}/packaging/scripts/source-tarball.sh" "${VERSION}" "${OUTDIR}")"

cp "${TARBALL}" "${TOPDIR}/SOURCES/"
cp "${REPO_ROOT}/packaging/fedora/lightkeeper.spec" "${TOPDIR}/SPECS/"

rpmbuild \
    --define "_topdir ${TOPDIR}" \
    -ba "${TOPDIR}/SPECS/lightkeeper.spec"

echo "RPM_DIR=${TOPDIR}/RPMS"
echo "SRPM_DIR=${TOPDIR}/SRPMS"
