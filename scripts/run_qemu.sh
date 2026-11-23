#!/usr/bin/env bash
# Mayaank N - Micro-OS QEMU Runner
# Student: Mayaank N (25BTRCL148)

set -eo pipefail

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

# Run QEMU under a bounded 10-second window to capture boot sequence and multi-cycle task switches.
# In continuous kernel execution, timeout exit codes 124 (timeout) or 143 (SIGTERM) indicate the intended test sampling window completed.
set +e
timeout 10s qemu-system-x86_64 \
    -cdrom mayaank_os.iso \
    -serial stdio \
    -display none \
    -m 256M \
    -no-reboot \
    > traces/qemu_boot_trace.log 2>&1
QEMU_STATUS=$?
set -e

if [ "$QEMU_STATUS" -eq 0 ] || [ "$QEMU_STATUS" -eq 124 ] || [ "$QEMU_STATUS" -eq 143 ]; then
    echo "[SUCCESS] QEMU execution completed sampling window (Status: ${QEMU_STATUS})."
    echo "[SUCCESS] QEMU execution trace saved in traces/qemu_boot_trace.log"
    echo ""
    echo "--- Trace Sample (First 20 lines) ---"
    head -n 20 traces/qemu_boot_trace.log
    echo "..."
    echo "--- Trace Sample (Last 15 lines) ---"
    tail -n 15 traces/qemu_boot_trace.log
    exit 0
else
    echo "[ERROR] QEMU failed unexpectedly with exit code ${QEMU_STATUS}!" >&2
    tail -n 25 traces/qemu_boot_trace.log >&2
    exit "$QEMU_STATUS"
fi
