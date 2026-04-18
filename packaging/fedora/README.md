# Fedora 43 packaging for Lightkeeper

## Prerequisites

Minimum **rustc 1.88** (see `rust-version` in the repo root `Cargo.toml` and `lightkeeper.spec`).
Check with `rustc --version`; Fedora 43 `rust` packages should meet this. If not, use
[rustup](https://rustup.rs/) and put `~/.cargo/bin` first on `PATH` for `rpmbuild`.

Install build dependencies (Fedora 43):

Either match everything in `packaging/fedora/lightkeeper.spec` in one step (recommended):

```bash
sudo dnf install dnf-plugins-core rpm-build rpmdevtools git
sudo dnf builddep packaging/fedora/lightkeeper.spec
```

Or install the same set explicitly:

```bash
sudo dnf install rust cargo gcc gcc-c++ make git \
    perl-File-Compare perl-File-Copy perl-FindBin perl-IPC-Cmd perl-Time-Piece \
    qt6-qtbase-devel qt6-qtdeclarative-devel \
    dbus-devel liboping-devel \
    rpm-build rpmdevtools
```

`rpmbuild` refuses to start if any `BuildRequires` package is missing, so after changing the spec run `dnf builddep` again.

## Source checkout

Clone this repository with submodules (required for `source-tarball.sh`):

```bash
git clone --recurse-submodules https://github.com/kalaksi/lightkeeper.git
cd lightkeeper
```

## Preparing the source archive

The source tarball needs to include all git submodules since the RPM build
runs without network access.

```bash
VERSION=$(packaging/scripts/version.sh)
TARBALL=$(packaging/scripts/source-tarball.sh "${VERSION}" packaging/fedora/tarballs)
echo "Source archive: ${TARBALL}"
```

## Building locally

```bash
VERSION=$(packaging/scripts/version.sh)
TOPDIR=packaging/fedora/build
OUTDIR=packaging/fedora/tarballs
packaging/fedora/scripts/build-binary.sh "${VERSION}" "${TOPDIR}" "${OUTDIR}"
```

The built RPMs will be in `packaging/fedora/build/RPMS/x86_64/`.

## Installing the built RPM

```bash
sudo dnf install packaging/fedora/build/RPMS/x86_64/lightkeeper-*.rpm
```

## Smoke testing

After installing, verify:
1. `lightkeeper` launches without errors.
2. No QML import errors in output (`RUST_LOG=debug lightkeeper`).
3. Terminal tab works (uses bundled qmltermwidget).
4. Charts view works (uses bundled ChartJs).
5. File browser works (uses bundled Lighthouse components).
6. Code editor loads (requires `qt6-qtwebengine`).
