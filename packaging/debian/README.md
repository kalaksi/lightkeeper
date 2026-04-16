# Debian Packaging for Lightkeeper

Debian 13 (Trixie) DEB packaging using GitHub Release DEB artifacts as the
primary distribution channel.

## Prerequisites

### Rust

Minimum **rustc 1.88** (see `rust-version` in the repo root `Cargo.toml` and `debian/control`).
Check with `rustc --version`. If the distro package is older, install a toolchain with
[rustup](https://rustup.rs/), for example:

```bash
curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.88 --profile minimal
. "$HOME/.cargo/env"
```

Put `~/.cargo/bin` on `PATH` when running `dpkg-buildpackage` so `cargo` matches that `rustc`.

### Other build dependencies

Install build dependencies (Debian 13 Trixie):
```bash
sudo apt install rustc cargo gcc g++ make git \
    qt6-base-dev qt6-declarative-dev \
    libdbus-1-dev liboping-dev libssl-dev \
    perl debhelper devscripts dpkg-dev
```

Omit `rustc` and `cargo` from `apt` if you use only rustup for the toolchain.

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

## GitHub Actions

Debian package automation is in `.github/workflows/package-debian.yml`.
The workflow builds the package inside `debian:trixie` and uploads DEB artifacts.
It installs **Rust 1.88** via rustup (not `apt` `rustc`), then runs the same build scripts as locally.

## GitHub Release artifacts

Built DEBs are attached to GitHub Releases.

```bash
gh release upload v${VERSION} "$BUILDDIR"/lightkeeper_${VERSION}-1_amd64.deb
```

## Release checklist

1. Update version in `packaging/debian/debian/changelog` (use `dch`).
2. Confirm `rustc --version` is **1.88** or newer on the build host (or use rustup as above).
3. Regenerate source archive with `source-tarball.sh`.
4. Build and smoke-test locally.
5. Tag the release and let CI build and attach the DEB artifact.
