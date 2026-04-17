# Debian Packaging for Lightkeeper

Debian 14 Forky DEB packaging using GitHub Release DEB artifacts as the primary
distribution channel.

## Prerequisites

Minimum **rustc 1.88** (see `rust-version` in the repo root `Cargo.toml` and `debian/control`).

Install build dependencies (Debian 14 Forky):

```bash
sudo apt install rustc cargo gcc g++ make git \
    qt6-base-dev qt6-declarative-dev \
    libdbus-1-dev liboping-dev libssl-dev \
    perl debhelper devscripts dpkg-dev
```

## Source checkout

Clone this repository with submodules (required for `source-tarball.sh`):

```bash
git clone --recurse-submodules https://github.com/kalaksi/lightkeeper.git
cd lightkeeper
```

## Shared packaging scripts

Reusable helper scripts are in `packaging/`:
- `scripts/version.sh` reads project version from `Cargo.toml`
- `scripts/source-tarball.sh` creates a source archive with submodules
- `debian/scripts/stage-debian-dir.sh` copies `packaging/debian/debian` to build root
- `debian/scripts/build-binary.sh` builds unsigned binary DEB packages
- `debian/scripts/build-source.sh` builds unsigned source package artifacts
- `debian/scripts/list-artifacts.sh` lists built package artifacts

## Preparing the source archive

The source tarball needs to include all git submodules since the DEB build
runs without network access.

```bash
VERSION=$(./packaging/scripts/version.sh)
./packaging/scripts/source-tarball.sh "$VERSION" packaging/debian/tarballs

echo "Source archive: packaging/debian/tarballs/lightkeeper-${VERSION}.tar.gz"
```

## Building locally

Since the `debian/` directory lives under `packaging/debian/` rather than
at the source root, it needs to be staged before building.

```bash
VERSION=$(./packaging/scripts/version.sh)
TARBALLDIR=packaging/debian/tarballs
BUILDDIR=packaging/debian/build
./packaging/debian/scripts/build-binary.sh "$VERSION" "$TARBALLDIR" "$BUILDDIR"
```

The built DEBs will be in `$BUILDDIR/`.

## Installing the built DEB

```bash
sudo apt install "$BUILDDIR"/lightkeeper_*.deb
```

## Smoke testing

After installing, verify:
1. `lightkeeper` launches without errors.
2. No QML import errors in output (`RUST_LOG=debug lightkeeper`).
3. Terminal tab works (uses bundled qmltermwidget).
4. Charts view works (uses bundled ChartJs).
5. File browser works (uses bundled Lighthouse components).
6. Code editor loads (requires `qml6-module-qtwebengine`).