# Mayaank N – Micro-OS

[![Rust](https://img.shields.io/badge/language-Rust%202021-orange.svg)](https://www.rust-lang.org/)
[![Architecture](https://img.shields.io/badge/architecture-x86__64-blue.svg)]()
[![Bootloader](https://img.shields.io/badge/bootloader-Limine%20v8-green.svg)](https://github.com/limine-bootloader/limine)
[![Status](https://img.shields.io/badge/tests-13%20passed-brightgreen.svg)]()

A bare-metal educational microkernel written in safe and systems-level Rust (`no_std`, `no_main`) featuring a specialized **Round Robin Scheduler with Dynamic Quantum Adaptation**. Developed as an individual Operating Systems CA6 Engineering Project.

---

## 👤 Student & Project Metadata

- **Student Name:** Mayaank N
- **USN:** 25BTRCL148
- **Project Title:** Rust Micro-OS Engineering Project
- **Course:** Operating System Concepts (CA6)
- **Specialization:** Scheduler Engineering
- **Project Variant:** Minimal Round Robin Micro-OS with Dynamic Quantum
- **Development Environment:** Ubuntu 24.04 WSL + VS Code + Antigravity

---

## 🌟 Key Features

1. **Bare-Metal x86_64 Kernel:** Real educational bare-metal kernel with no user-space emulation or standard library dependencies.
2. **Limine Boot Protocol (v8):** Modern 64-bit bootloader integration with Higher Half Direct Map (HHDM) support.
3. **Structured Dual Console:** Simultaneous output to 16550 UART Serial (`COM1 0x3F8`) and VGA text buffer (`0xb8000`).
4. **Specialized Dynamic Quantum Scheduler:** Adapts task time quanta ($Q_{dyn} \in [2, 12]$ ticks) dynamically based on recent CPU burst history and ready queue load.
5. **Interrupt & Hardware Time Base:** Programmable Interval Timer (PIT 8254) configured at 100 Hz (10 ms ticks), dual 8259 PIC remapping (vectors 32–47), and a 256-entry Interrupt Descriptor Table (IDT).
6. **Deadlock-Free Spinlocks:** Hardware atomic spinlock (`SpinLock<T>`) with automatic interrupt disabling (`lock_irq_disabled`) to prevent deadlocks in ISRs.
7. **Linear Bump Memory Allocator:** Static 2 MiB kernel heap managing 16-byte aligned allocations (`kmalloc`).
8. **Interactive Companion Simulator:** Standalone Streamlit application ready for deployment on Hugging Face Spaces.

---

## 📁 Repository Structure

```
mayaank-micro-os/
├── Cargo.toml                  # Cargo workspace configuration
├── limine.conf                 # Limine bootloader configuration
├── kernel/                     # Bare-metal kernel package (x86_64-unknown-none)
│   ├── Cargo.toml
│   ├── linker.ld               # ELF64 Higher-half linker script
│   └── src/
│       ├── main.rs             # Kernel entry point (_start) and controlled tasks
│       ├── drivers/            # UART Serial, VGA, 8259 PIC, 8254 PIT, Port I/O
│       ├── sync/               # Atomic SpinLock and interrupt-safe guards
│       ├── memory/             # BumpAllocator and 2 MiB static heap
│       ├── task/               # TaskId, TaskState, TaskContext, TaskControlBlock
│       ├── scheduler/          # ReadyQueue, DynamicQuantumEngine, RoundRobinScheduler
│       ├── timer/              # Hardware tick tracking, IDT, exception handlers
│       └── kernel/             # Limine bindings, logging macros, diagnostics
├── scheduler_sim/              # Host-side simulator and unit test verification package
├── huggingface_space/          # Interactive Streamlit companion for Hugging Face Spaces
├── scripts/
│   ├── build_iso.sh            # Compiles kernel and builds bootable Limine ISO
│   ├── run_qemu.sh             # Launches QEMU in headless serial mode
│   └── test_all.sh             # Master test runner (unit tests + build + QEMU)
├── traces/                     # Real, verified execution traces and logs
│   ├── qemu_boot_trace.log     # Raw serial capture of live QEMU execution
│   └── qemu_execution_trace.txt# Full multi-cycle scheduler trace (9,200+ switches)
├── results/                    # Automated test outputs and build logs
├── docs/                       # Technical documentation and reports
│   ├── architecture.md         # System architecture and Mermaid diagrams
│   ├── design.md               # Design decisions, trade-offs, and dynamic quantum model
│   ├── testing.md              # Test execution instructions and verification matrix
│   ├── debugging-log.md        # Technical incident log and bug resolutions
│   ├── performance-analysis.md # Fixed RR vs Dynamic RR empirical benchmarks
│   ├── limitations.md          # Architectural boundaries and future engineering
│   ├── report.md               # Comprehensive formal academic technical report
│   └── viva-qa.md              # Comprehensive viva defense questions and answers
└── tools/                      # Built-in Limine bootloader binaries
```

---

## 🚀 Quick Start & Build Instructions

### Prerequisites
- **Rust Toolchain:** `nightly` with `x86_64-unknown-none` target:
  ```bash
  rustup target add x86_64-unknown-none
  ```
- **System Utilities:** `qemu-system-x86_64`, `xorriso`, `mtools`.

### 1. Run Complete Automated Verification Suite
Execute all host unit tests, compile the bare-metal kernel, generate the bootable ISO, and boot in QEMU:
```bash
chmod +x scripts/test_all.sh
./scripts/test_all.sh
```

### 2. Build the Bootable ISO
```bash
chmod +x scripts/build_iso.sh
./scripts/build_iso.sh
```
This produces `mayaank_os.iso`.

### 3. Run in QEMU
```bash
chmod +x scripts/run_qemu.sh
./scripts/run_qemu.sh
```
Or run QEMU directly:
```bash
qemu-system-x86_64 -cdrom mayaank_os.iso -serial stdio -display none
```

### 4. Run Host Unit Tests
```bash
cargo test --package scheduler_sim --target x86_64-unknown-linux-gnu
```

---

## 📊 Live Verification Evidence

During live execution in QEMU (captured in `traces/qemu_execution_trace.txt`):
- **Context Switches:** Executed **9,209** context switches seamlessly across 868 timer ticks.
- **Round-Robin Cycles:** Completed **3,071** round-robin cycles among Task 1 (`Compute-Worker`), Task 2 (`Telemetry-Worker`), and Task 3 (`Burst-Worker`).
- **Dynamic Quantum Adaptation:** Verified slice expansion for compute workloads (up to 10 ticks) and compression for yielding bursts (2 ticks).
- **Memory Allocation:** 98,304 bytes (96 KiB) allocated for 3 task stacks from the bump allocator.
- **Fault Rate:** 0 General Protection Faults, 0 Double Faults, 0 Panics.

---

## 🤗 Hugging Face Spaces Companion

A separate interactive companion scheduler simulator is located in `huggingface_space/`:
- **Proposed Space Name:** `mayaank-n-micro-os-scheduler`
- **SDK:** Streamlit
- **Features:** Interactive task configurator, Gantt execution timeline, live queue state inspection, and comparative telemetry (Fixed vs. Dynamic RR).
- **Run locally:**
  ```bash
  cd huggingface_space
  pip install -r requirements.txt
  streamlit run app.py
  ```

---

## 📜 Academic Integrity & License

This project was developed by **Mayaank N (USN: 25BTRCL148)** for the CA6 Operating Systems Engineering Coursework.  
Distributed under the MIT License.
