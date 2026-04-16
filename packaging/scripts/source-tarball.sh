#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
    echo "usage: $0 <version> <output-dir>" >&2
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)

VERSION="$1"
OUTDIR="$(mkdir -p "$2" && cd "$2" && pwd)"

cd "${REPO_ROOT}"
git submodule update --init --recursive >&2

git archive --prefix="lightkeeper-${VERSION}/" HEAD \
    -o "${OUTDIR}/lightkeeper-base.tar"

git submodule foreach --recursive \
    "rel_path=\$(realpath --relative-to=\"${REPO_ROOT}\" \"\$toplevel/\$sm_path\"); \
     git archive --prefix=lightkeeper-${VERSION}/\$rel_path/ HEAD \
     -o ${OUTDIR}/lightkeeper-sub-\$(echo \$rel_path | tr / -).tar" >&2

cp "${OUTDIR}/lightkeeper-base.tar" "${OUTDIR}/lightkeeper-${VERSION}.tar"

for sub in "${OUTDIR}"/lightkeeper-sub-*.tar; do
    tar --concatenate --file="${OUTDIR}/lightkeeper-${VERSION}.tar" "${sub}"
done

gzip -f "${OUTDIR}/lightkeeper-${VERSION}.tar"

echo "${OUTDIR}/lightkeeper-${VERSION}.tar.gz"
