#!/usr/bin/env bash
# Mayaank N - Micro-OS Master Test Suite Runner
# Student: Mayaank N (25BTRCL148)

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"
mkdir -p results traces

echo "================================================================================"
echo "          MAYAANK N - MICRO-OS AUTOMATED VERIFICATION SUITE                    "
echo "================================================================================"
echo "Student: Mayaank N (25BTRCL148)"
echo "Target Architecture: x86_64-unknown-none"
echo "Host Environment: Ubuntu 24.04 WSL"
echo "Date: $(date)"
echo "================================================================================"
echo ""

source ~/.cargo/env

echo "[STEP 1/4] Running Automated Unit Tests (Scheduler, Tasks, Allocator, Sync)..."
cargo test --package scheduler_sim --lib --target x86_64-unknown-linux-gnu | tee results/unit_tests.log

echo ""
echo "[STEP 2/4] Compiling Bare-Metal Rust Kernel (no_std, no_main)..."
cargo build --package mayaank-micro-os --target x86_64-unknown-none | tee results/kernel_build.log

echo ""
echo "[STEP 3/4] Building Limine Bootable ISO..."
./scripts/build_iso.sh | tee results/iso_build.log

echo ""
echo "[STEP 4/4] Executing QEMU Integration Test (Headless Serial Capture)..."
./scripts/run_qemu.sh | tee results/qemu_integration.log

echo ""
echo "================================================================================"
echo "          ALL TESTS AND VERIFICATION PHASES COMPLETED SUCCESSFULLY!             "
echo "================================================================================"
