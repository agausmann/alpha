#!/bin/sh

set -e

cleanup() {
    [ -n "$MOUNT" ] && sudo umount "$MOUNT"
    [ -n "$LOOP" ] && sudo losetup -d "$LOOP"
}
trap cleanup EXIT INT HUP TERM

LOOP="$(sudo losetup -P -f --show vm/disk.img)"
sudo mount "${LOOP}p1" /mnt
MOUNT=/mnt

cd codegen
cargo run
sudo cp kernel.elf /mnt/