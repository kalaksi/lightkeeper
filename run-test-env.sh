#!/usr/bin/env bash
set -eu
RUST_LOG=debug ./target/debug/lightkeeper --config-dir test-env
# flatpak run --filesystem=~/git/lightkeeper/test-env io.github.kalaksi.Lightkeeper --config-dir test-env
