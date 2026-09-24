#!/usr/bin/env bash
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
VM_NAME=amazonlinux2023
URI=qemu:///system
POOL_NAME=default
VOLUME_NAME="$VM_NAME.qcow2"

export LANG=C LC_ALL=C
VIRSH=(virsh -c "$URI")

if "${VIRSH[@]}" dominfo "$VM_NAME" >/dev/null 2>&1; then
    "${VIRSH[@]}" destroy "$VM_NAME" 2>/dev/null || true
    "${VIRSH[@]}" undefine "$VM_NAME" --nvram 2>/dev/null || "${VIRSH[@]}" undefine "$VM_NAME"
fi

rm -f "$SCRIPT_DIR/disk.qcow2" "$SCRIPT_DIR/$VM_NAME".*.qcow2

if "${VIRSH[@]}" vol-info --pool "$POOL_NAME" "$VOLUME_NAME" >/dev/null 2>&1; then
    "${VIRSH[@]}" vol-delete --pool "$POOL_NAME" "$VOLUME_NAME"
fi
