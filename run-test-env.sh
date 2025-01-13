#!/usr/bin/env bash
set -eu
RUST_LOG=debug ./target/debug/lightkeeper --config-dir test-env
