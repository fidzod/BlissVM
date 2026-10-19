#!/usr/bin/env bash
# boot.sh - build everything and boot the Bliss VM.
# Compiles the Rust toolchain, assembles the kernel and user programs,
# writes a BlissFS disk image, then launches the VM.

set -eo pipefail

run() { "$@" 2>&1 | sed 's/^/[boot.sh] /'; }

run cargo build --quiet

run target/debug/asm os/ikari.asm   -pic
run target/debug/asm os/init.asm    -pic
run target/debug/asm os/kernel.asm

run target/debug/mkfs disk.img os/init.bin os/ikari.bin

echo "[boot.sh] Starting..."
target/debug/bliss os/kernel.bin disk.img
