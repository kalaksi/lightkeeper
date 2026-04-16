#!/usr/bin/env bash
set -euo pipefail

sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1
