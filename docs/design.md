# System Design Document

**Project Name:** Mayaank N – Micro-OS  
**Student Name:** Mayaank N  
**USN:** 25BTRCL148  
**Specialization:** Scheduler Engineering – Round Robin with Dynamic Quantum  

---

## 1. Design Decisions & Trade-Offs

### 1.1 Programming Language: Bare-Metal Rust (`no_std`, `no_main`)
- **Decision:** Use Rust without the standard library instead of C or C++.
- **Rationale:** 
  - Rust guarantees memory safety at compile time, eliminating buffer overflows, use-after-free, and dangling pointers.
  - Pattern matching and strong type abstractions (`TaskId`, `TaskState`, `ColorCode`) prevent invalid state assignments.
  - Zero-cost abstractions allow high-level modular code without runtime performance penalties.
- **Trade-off:** Requires `unsafe` blocks for direct hardware memory manipulation, port I/O (`core::arch::asm!`), and naked functions for low-level context switching.

### 1.2 Bootloader: Limine Boot Protocol v8
- **Decision:** Use Limine instead of custom 16-bit real-mode assembly bootloaders or legacy GRUB.
- **Rationale:**
  - Limine establishes 64-bit long mode, paging, and a Higher Half Direct Map (HHDM) before handing control to `_start`.
  - Modern protocol with clear request-response handshake (`LIMINE_HHDM_REQUEST`, `LIMINE_FRAMEBUFFER_REQUEST`).
  - Eliminates fragile real-mode to protected-mode transition boilerplate, focusing development on actual OS engineering.

### 1.3 Synchronization: Deadlock-Free Interrupt-Safe Spinlock
- **Decision:** Implement `SpinLock<T>` with both standard `lock()` and `lock_irq_disabled()`.
- **Rationale:**
  - If a thread acquires a spinlock while interrupts are enabled, and a timer interrupt fires on the same CPU, the timer ISR trying to acquire the same spinlock will spin forever, causing an unrecoverable kernel deadlock.
  - `lock_irq_disabled()` reads `rflags`, executes `cli`, acquires the atomic lock, and on drop automatically restores previous interrupt flags.

### 1.4 Memory Management: Linear Bump Allocator
- **Decision:** Implement a bounded `BumpAllocator` over a static 2 MiB BSS heap buffer.
- **Rationale:**
  - Bump allocation offers $O(1)$ constant time allocation with zero metadata overhead and zero heap fragmentation during system startup.
  - Statically backed heap ensures the kernel never depends on external physical frame mappings before the scheduler is active.
- **Trade-off:** Memory cannot be individually freed; it is designed for persistent kernel objects (TCBs, stacks, IDT buffers).

---

## 2. Dynamic Quantum Scheduler Design (Specialization)

### 2.1 The Dynamic Quantum Problem
In standard Round Robin (RR), the time quantum $Q$ is static:
- If $Q$ is too small: High context-switching overhead, cache thrashing, and low CPU throughput.
- If $Q$ is too large: High average waiting time, poor interactive responsiveness, approaching First-Come-First-Served (FCFS).

### 2.2 Mathematical Model
The dynamic quantum algorithm continuously recalibrates task time slices based on real-time execution feedback:

$$Q_{dyn}(T) = \text{clamp}\left( Q_{base} + \Delta_{burst} - \Delta_{load} + \Delta_{priority}, Q_{min}, Q_{max} \right)$$

Where:
- $Q_{base} = 4\text{ ticks}$ (40 ms baseline)
- $Q_{min} = 2\text{ ticks}$ (20 ms minimum bound)
- $Q_{max} = 12\text{ ticks}$ (120 ms maximum bound)
- **Burst Adaptation ($\Delta_{burst}$):**
  $$\Delta_{burst} = \begin{cases} +1 & \text{if } \text{burst}_{avg} \ge Q_{assigned} \text{ (CPU-bound workload)} \\ -1 & \text{if } \text{burst}_{avg} \le Q_{min} \text{ (Interactive / Yielding)} \\ 0 & \text{otherwise} \end{cases}$$
- **Queue Pressure Penalty ($\Delta_{load}$):**
  $$\Delta_{load} = \min(2, \max(0, |Q_{ready}| - 3))$$
- **Priority Scaling ($\Delta_{priority}$):**
  $$\Delta_{priority} = \lfloor \text{priority} / 2 \rfloor$$

---

## 3. Low-Level Context Switch Design

Context switching is performed via a naked assembly routine `switch_context(old_ctx, new_ctx)`:
1. Pushes callee-saved registers (`rbp`, `rbx`, `r12`, `r13`, `r14`, `r15`) onto the current task's stack.
2. Stores the updated stack pointer into `old_ctx.rsp`.
3. Loads the destination task's stack pointer from `new_ctx.rsp` into `rsp`.
4. Pops callee-saved registers (`r15`, `r14`, `r13`, `r12`, `rbx`, `rbp`) from the new stack.
5. Executes `ret`, which pops the return address (`rip`) and resumes the target task seamlessly.

Initial task stacks are seeded with a simulated frame matching this exact layout, allowing `ret` to jump cleanly into the task entry function on its first dispatch.
