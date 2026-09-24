#!/usr/bin/env bash
set -eu

VM_NAME=amazonlinux2023
URI=qemu:///system

export LANG=C LC_ALL=C
VIRSH=(virsh -c "$URI")

if ! "${VIRSH[@]}" dominfo "$VM_NAME" >/dev/null 2>&1; then
    exit 0
fi

if "${VIRSH[@]}" list --state-running --name | grep -qx "$VM_NAME"; then
    "${VIRSH[@]}" shutdown "$VM_NAME" || "${VIRSH[@]}" destroy "$VM_NAME"
fi
