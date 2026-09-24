#!/usr/bin/env bash
set -eu

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
VM_NAME=amazonlinux2023
URI=qemu:///system
NETWORK_NAME=vagrant-libvirt
POOL_NAME=default
VOLUME_NAME="$VM_NAME.qcow2"
BASE_IMAGE="$SCRIPT_DIR/al2023-kvm.qcow2"
SEED_DIR="$SCRIPT_DIR/seed"
SSH_KEY="$SCRIPT_DIR/id_ed25519"
SSH_PUBKEY="$SCRIPT_DIR/id_ed25519.pub"
IMAGE_INDEX_URL=https://cdn.amazonlinux.com/al2023/os-images/latest/kvm/

export LANG=C LC_ALL=C
VIRSH=(virsh -c "$URI")

if [[ ! -f "$SSH_KEY" ]]; then
    ssh-keygen -t ed25519 -N "" -f "$SSH_KEY" -C "$VM_NAME"
fi

if [[ ! -e "$BASE_IMAGE" ]]; then
    listing=$(curl -fsSL "$IMAGE_INDEX_URL")
    image_name=$(printf '%s\n' "$listing" | grep -oE 'al2023-kvm-[^"<> ]+-x86_64\.xfs\.gpt\.qcow2' | head -n1)
    if [[ -z "$image_name" ]]; then
        echo "Could not find AL2023 KVM image under $IMAGE_INDEX_URL" >&2
        exit 1
    fi
    if [[ ! -f "$SCRIPT_DIR/$image_name" ]]; then
        echo "Downloading $image_name..."
        curl -fL --progress-bar -o "$SCRIPT_DIR/$image_name.partial" "$IMAGE_INDEX_URL$image_name"
        mv "$SCRIPT_DIR/$image_name.partial" "$SCRIPT_DIR/$image_name"
    fi
    ln -sfn "$image_name" "$BASE_IMAGE"
fi

mkdir -p "$SEED_DIR"
cat > "$SEED_DIR/meta-data" <<EOF
instance-id: $VM_NAME
local-hostname: $VM_NAME
EOF
cat > "$SEED_DIR/user-data" <<EOF
#cloud-config
users:
  - default
  - name: ec2-user
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - $(cat "$SSH_PUBKEY")
ssh_pwauth: false
EOF
rm -f "$SEED_DIR/network-config"

if ! "${VIRSH[@]}" pool-info "$POOL_NAME" >/dev/null 2>&1; then
    echo "Storage pool '$POOL_NAME' not found on $URI." >&2
    exit 1
fi
"${VIRSH[@]}" pool-start "$POOL_NAME" >/dev/null 2>&1 || true
"${VIRSH[@]}" pool-refresh "$POOL_NAME" >/dev/null

# Remove leftover project-dir disk from earlier attempts.
rm -f "$SCRIPT_DIR/disk.qcow2"

if ! "${VIRSH[@]}" vol-info --pool "$POOL_NAME" "$VOLUME_NAME" >/dev/null 2>&1; then
    echo "Creating libvirt volume $POOL_NAME/$VOLUME_NAME..."
    base_path=$(readlink -f "$BASE_IMAGE")
    virtual_bytes=$(qemu-img info --output=json -f qcow2 "$base_path" \
        | grep -o '"virtual-size": [0-9]*' | head -n1 | grep -o '[0-9]*')
    tmp_disk=$(mktemp "$SCRIPT_DIR/$VM_NAME.XXXX.qcow2")
    trap 'rm -f "$tmp_disk"' EXIT
    qemu-img convert -p -f qcow2 -O qcow2 "$base_path" "$tmp_disk"
    "${VIRSH[@]}" vol-create-as "$POOL_NAME" "$VOLUME_NAME" "${virtual_bytes}b" --format qcow2
    "${VIRSH[@]}" vol-upload --pool "$POOL_NAME" "$VOLUME_NAME" "$tmp_disk"
    rm -f "$tmp_disk"
    trap - EXIT
fi

DISK_IMAGE=$("${VIRSH[@]}" vol-path --pool "$POOL_NAME" "$VOLUME_NAME")

if ! "${VIRSH[@]}" net-info "$NETWORK_NAME" >/dev/null 2>&1; then
    echo "Network '$NETWORK_NAME' not found on $URI (used by Vagrant)." >&2
    exit 1
fi
"${VIRSH[@]}" net-start "$NETWORK_NAME" >/dev/null 2>&1 || true

if ! "${VIRSH[@]}" dominfo "$VM_NAME" >/dev/null 2>&1; then
    virt-install \
        --connect "$URI" \
        --name "$VM_NAME" \
        --memory 2048 \
        --vcpus 2 \
        --disk "path=$DISK_IMAGE,format=qcow2,bus=virtio" \
        --network "network=$NETWORK_NAME,model=virtio" \
        --osinfo detect=on,require=off \
        --virt-type kvm \
        --graphics none \
        --serial pty \
        --console pty,target_type=serial \
        --import \
        --noautoconsole \
        --cloud-init "user-data=$SEED_DIR/user-data,meta-data=$SEED_DIR/meta-data"
fi

"$SCRIPT_DIR/start.sh"

guest_mac=$("${VIRSH[@]}" domiflist "$VM_NAME" 2>/dev/null \
    | awk 'NR>2 && $5 ~ /:/ { print $5; exit }' || true)
guest_ip=""
if [[ -n "$guest_mac" ]]; then
    deadline=$((SECONDS + 60))
    while (( SECONDS < deadline )); do
        guest_ip=$("${VIRSH[@]}" net-dhcp-leases "$NETWORK_NAME" 2>/dev/null \
            | awk -v mac="$guest_mac" 'BEGIN { IGNORECASE = 1 } $3 == mac {
                split($5, parts, "/")
                print parts[1]
                exit
            }' || true)
        [[ -n "$guest_ip" ]] && break
        sleep 2
    done
fi

echo "VM $VM_NAME defined on $URI / $NETWORK_NAME."
echo "Disk:    $POOL_NAME/$VOLUME_NAME"
echo "SSH key: $SSH_KEY"
echo "User:    ec2-user"
if [[ -n "$guest_ip" ]]; then
    echo "Address: $guest_ip (port 22)"
else
    echo "Address: (DHCP pending) run: virsh -c $URI net-dhcp-leases $NETWORK_NAME"
fi
echo "Console: virsh -c $URI console $VM_NAME"
