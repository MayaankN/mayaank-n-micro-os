#!/usr/bin/env bash
# Mayaank N - Micro-OS QEMU Runner
# Student: Mayaank N (25BTRCL148)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

mkdir -p traces results

if [ ! -f mayaank_os.iso ]; then
    echo "[RUN] ISO not found. Building first..."
    ./scripts/build_iso.sh
fi

echo "[RUN] Launching Mayaank Micro-OS in QEMU (Headless Serial Mode)..."
echo "[RUN] Capturing execution trace into traces/qemu_boot_trace.log..."

# Run QEMU with timeout of 5 seconds to capture boot sequence and task context switches
timeout --preserve-status 10s qemu-system-x86_64 \
    -cdrom mayaank_os.iso \
    -serial stdio \
    -display none \
    -m 256M \
    -no-reboot \
    | tee traces/qemu_boot_trace.log

echo "[SUCCESS] QEMU execution trace saved in traces/qemu_boot_trace.log"
