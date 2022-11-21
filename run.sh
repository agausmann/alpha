#!/bin/sh

QEMU=qemu-system-x86_64

cd vm

"$QEMU" \
    -drive file=disk.img,format=raw \
    -display sdl