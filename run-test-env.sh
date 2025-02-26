#!/usr/bin/env bash
set -eu
# RUST_LOG=debug ./target/debug/lightkeeper --config-dir test-env
RUST_LOG=debug flatpak run --filesystem=~/git/lightkeeper/test-env io.github.kalaksi.Lightkeeper-local --config-dir test-env
