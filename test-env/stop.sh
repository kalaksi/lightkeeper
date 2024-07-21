#!/usr/bin/env bash
set -eu

current_dir=$(dirname "$0")

for vagrantfile in $current_dir/**/Vagrantfile; do
    dir="$(dirname "$vagrantfile")"
    pushd "$dir" && vagrant halt --no-tty && popd
done
