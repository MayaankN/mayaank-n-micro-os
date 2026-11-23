#!/usr/bin/env bash
# Mayaank N - Micro-OS ISO Builder
# Student: Mayaank N (25BTRCL148)

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

echo "[BUILD] Compiling Mayaank Micro-OS kernel (x86_64-unknown-none)..."
source ~/.cargo/env
cargo build --package mayaank-micro-os --target x86_64-unknown-none

echo "[BUILD] Preparing ISO filesystem structure..."
rm -rf iso_root mayaank_os.iso
mkdir -p iso_root/EFI/BOOT

cp target/x86_64-unknown-none/debug/mayaank-micro-os iso_root/kernel.elf
cp limine.conf iso_root/limine.conf

cp tools/limine/limine-bios.sys iso_root/
cp tools/limine/limine-bios-cd.bin iso_root/
cp tools/limine/limine-uefi-cd.bin iso_root/
cp tools/limine/BOOTX64.EFI iso_root/EFI/BOOT/
cp tools/limine/BOOTIA32.EFI iso_root/EFI/BOOT/

echo "[BUILD] Generating bootable ISO using xorriso..."
xorriso -as mkisofs -b limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        iso_root -o mayaank_os.iso

echo "[BUILD] Installing Limine BIOS bootloader onto ISO..."
./tools/limine/limine bios-install mayaank_os.iso

echo "[SUCCESS] Mayaank Micro-OS bootable ISO created: mayaank_os.iso"
