#!/usr/bin/env bash
set -eu

enabled_vms=("centos7" "centos8" "debian10" "debian11" "ubuntu2004")

for vm in "${enabled_vms[@]}"; do
    if [ -d "test/$vm" ]; then
        cd "test/$vm"
        vagrant up --no-tty
        cd -
    fi
done
 
 RUST_LOG=debug ./target/debug/lightkeeper --config-dir test

for vm in "${enabled_vms[@]}"; do
    if [ -d "test/$vm" ]; then
        cd "test/$vm"
        vagrant halt --no-tty
        cd -
    fi
done
