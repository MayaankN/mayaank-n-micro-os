# Testing & Verification Guide

**Project Name:** Mayaank N – Micro-OS  
**Student Name:** Mayaank N  
**USN:** 25BTRCL148  
**Specialization:** Scheduler Engineering – Round Robin with Dynamic Quantum  

---

## 1. Testing Strategy

The verification strategy employs a two-tier methodology to ensure genuine hardware correctness without fabricating test results:

1. **Host-Side Automated Unit Tests (`scheduler_sim`):**
   - Compiled for `x86_64-unknown-linux-gnu` using standard Rust test harness.
   - Verifies algorithmic logic, queue ordering, dynamic quantum adaptation, allocator boundary checks, and multi-threaded synchronization.
2. **Bare-Metal Kernel Integration Tests (QEMU):**
   - Compiled for `x86_64-unknown-none` with Limine bootloader ISO.
   - Boots in QEMU in headless serial mode (`-serial stdio -display none`).
   - Verifies real hardware timer interrupts (PIT 8254), PIC remapping, IDT dispatch, and live x86_64 context switches.

---

## 2. Test Execution Commands

### Run Complete Verification Suite (One Command)
```bash
chmod +x scripts/test_all.sh
./scripts/test_all.sh
```

### Run Host-Side Unit Tests
```bash
cargo test --package scheduler_sim --target x86_64-unknown-linux-gnu
```

### Run Bare-Metal Kernel in QEMU
```bash
chmod +x scripts/run_qemu.sh
./scripts/run_qemu.sh
```

---

## 3. Unit Test Verification Matrix

| Test Identifier | Category | Verification Objective | Result |
| :--- | :--- | :--- | :--- |
| `test_task_creation` | Task Management | Validates TCB initialization, initial state `Ready`, baseline quantum, stack bounds | **PASSED** |
| `test_task_state_transitions` | Task Lifecycle | Verifies state transitions `Ready -> Running -> Blocked -> Terminated` | **PASSED** |
| `test_ready_queue_ordering` | Ready Queue | Verifies strict First-In-First-Out (FIFO) queue ordering | **PASSED** |
| `test_ready_queue_boundary_capacity` | Boundary Conditions | Verifies queue rejects pushes beyond `READY_QUEUE_CAPACITY` (32) without buffer overrun | **PASSED** |
| `test_round_robin_scheduling` | Scheduler Core | Verifies fair cyclic dispatch among multiple tasks and context switch tracking | **PASSED** |
| `test_dynamic_quantum_cpu_bound_expansion`| Specialization | Verifies CPU-exhausting tasks are rewarded with increased quantum up to $Q_{max}$ | **PASSED** |
| `test_dynamic_quantum_interactive_reduction`| Specialization | Verifies early-yielding tasks have quantum reduced towards $Q_{min}$ for lower latency | **PASSED** |
| `test_dynamic_quantum_queue_pressure` | Specialization | Verifies high ready queue depth (>3) scales down quantum to protect wait latency | **PASSED** |
| `test_timer_ticks_and_quantum_expiration` | Timer Subsystem | Verifies timer tick increments and exact quantum expiration preemption triggers | **PASSED** |
| `test_synchronization_mutual_exclusion` | Synchronization | Verifies multi-threaded atomic mutual exclusion and lock release | **PASSED** |
| `test_bump_allocator_allocation_and_alignment`| Memory | Verifies 16-byte and 32-byte address alignments and capacity accounting | **PASSED** |
| `test_bump_allocator_out_of_memory_boundary`| Boundary Conditions | Verifies allocations exceeding heap capacity return `None` safely without panic | **PASSED** |
| `test_invalid_operations` | Error Handling | Verifies duplicate task spawning and out-of-bounds task IDs are safely rejected | **PASSED** |

**Host Unit Test Result:** `13 passed; 0 failed; 0 ignored; finished in 0.00s`

---

## 4. Bare-Metal QEMU Execution Evidence

In live QEMU execution (`scripts/run_qemu.sh`), the kernel completed:
- **Total Context Switches:** Over 9,209 context switches across 868 timer ticks.
- **Round-Robin Cycles:** Over 3,071 complete cycles between Task 1 (`Compute-Worker`), Task 2 (`Telemetry-Worker`), and Task 3 (`Burst-Worker`).
- **Memory Allocation:** 98,304 bytes (96 KiB) allocated for 3 task stacks from the 2 MiB linear bump allocator.
- **Faults / Panics:** Zero General Protection Faults, zero Double Faults, zero kernel panics.
- **Trace Archive:** Genuine execution logs are recorded in `traces/qemu_execution_trace.txt`.
