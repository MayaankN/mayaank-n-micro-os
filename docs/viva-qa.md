# Operating Systems CA6 Viva Defense Guide: Questions & Answers

**Project Title:** Mayaank N – Micro-OS: Round Robin Microkernel with Dynamic Quantum  
**Student Name:** Mayaank N  
**USN:** 25BTRCL148  
**Specialization:** Scheduler Engineering  

---

## Part 1: Operating System & Scheduler Core

### Q1: What is Round Robin (RR) scheduling, and what is its primary advantage and disadvantage?
**Answer:**  
Round Robin is a preemptive time-sharing scheduling algorithm that allocates a fixed time quantum ($Q$) to each process in a circular First-In-First-Out (FIFO) ready queue.  
- **Advantage:** Fairness and starvation-freedom. Every runnable task is guaranteed CPU execution within $(N - 1) \times Q$ time units.
- **Disadvantage:** Sensitivity to quantum size. If $Q$ is too small, excessive context switching degrades CPU throughput. If $Q$ is too large, it degenerates into First-Come-First-Served (FCFS), leading to poor interactive latency.

### Q2: What is your project's specialization, and how does it solve the fixed-quantum dilemma?
**Answer:**  
My specialization is **Dynamic Time Quantum Adaptation**. Instead of locking all tasks to a static slice, my `DynamicQuantumEngine` calculates an adapted quantum $Q_{dyn}$ for each task before placing it back in the ready queue:
1. **CPU-bound tasks** that exhaust their assigned quantum are given a larger quantum ($+1$ tick, up to 12 ticks) to reduce context switch overhead and boost throughput.
2. **Interactive tasks** that yield early before exhausting their slice have their quantum reduced towards 2 ticks, giving them quick, low-latency access when they have brief bursts.
3. **Queue load balancing:** When the ready queue depth exceeds 3 tasks, a penalty scales down quanta to prevent long waiting times.

### Q3: What is a context switch, and how is it implemented in your kernel?
**Answer:**  
A context switch is the process of storing the CPU state of a running task and restoring the state of the next task so that both can execute concurrently without interfering.  
In Mayaank N – Micro-OS, context switching is implemented in naked assembly in `switch_context(old_ctx, new_ctx)`:
1. It pushes x86_64 callee-saved registers (`rbp`, `rbx`, `r12`, `r13`, `r14`, `r15`) onto the outgoing task's stack.
2. It saves the outgoing `rsp` into `old_ctx.rsp`.
3. It loads the incoming `rsp` from `new_ctx.rsp`.
4. It pops the callee-saved registers from the incoming stack.
5. It executes `ret`, which pops the saved return address (`rip`) and resumes the target task.

### Q4: How is a new task's stack initialized before its first context switch?
**Answer:**  
When `TaskControlBlock::new()` creates a task, `TaskContext::init_stack()` creates a fake stack frame at `stack_top`:
- At `stack_top - 8`: The task's entry point address (which `ret` will pop).
- At `stack_top - 16` to `stack_top - 56`: Six zero-initialized qwords for `r15..rbp`.
- The TCB's `rsp` is set to `stack_top - 56`.  
When `switch_context` switches into this new task for the first time, it pops the six dummy registers and `ret` jumps cleanly into the task function.

---

## Part 2: Rust Systems Programming & Architecture

### Q5: Why build an operating system in Rust instead of C?
**Answer:**  
1. **Memory Safety:** Rust prevents buffer overflows, use-after-free, and dangling pointers at compile time through its strict ownership and borrowing rules.
2. **Concurrency Safety:** Rust's `Send` and `Sync` traits prevent race conditions on shared kernel resources.
3. **Type Safety:** Strongly typed abstractions like `TaskId(u64)` and `TaskState` prevent invalid state transitions.
4. **Zero-Cost Abstractions:** High-level constructs like iterators and RAII guards compile down to machine code with zero runtime overhead.

### Q6: What does `#![no_std]` and `#![no_main]` mean in your kernel?
**Answer:**  
- `#![no_std]` tells the Rust compiler not to link the standard library (`std`), which requires an underlying host operating system (like Linux or Windows). Instead, the kernel links only `core`, which contains compiler primitives without OS dependencies.
- `#![no_main]` disables the standard C runtime startup routine (`crt0`) and default `fn main()`, allowing us to define our own bare-metal entry point `#[no_mangle] pub extern "C" fn _start() -> !`.

### Q7: What is a Spinlock, and why did you implement `lock_irq_disabled()`?
**Answer:**  
A spinlock is a synchronization primitive where a thread repeatedly polls an atomic flag (`AtomicBool::swap`) until it becomes available.  
`lock_irq_disabled()` is essential for preventing deadlocks with interrupt handlers:
If a task acquires a spinlock with interrupts enabled, and an IRQ 0 timer interrupt fires on the same CPU, the timer ISR trying to acquire the same spinlock will loop forever because the interrupted task cannot run to release it. `lock_irq_disabled()` disables interrupts (`cli`), acquires the lock, and upon dropping the RAII guard restores the previous interrupt flag.

---

## Part 3: Hardware Interrupts & Memory Management

### Q8: How does the timer trigger scheduling in your kernel?
**Answer:**  
The 8254 PIT (Programmable Interval Timer) is configured at 100 Hz (10 ms ticks). On each tick, the PIT asserts IRQ 0. The 8259 PIC translates IRQ 0 into interrupt vector 32. The CPU checks the IDT at entry 32 and jumps to `timer_interrupt_entry`, which calls `timer_handler_inner()`. The handler increments the system tick counter, signals EOI to the PIC, and calls `SCHEDULER.on_timer_tick()`. If the task's quantum reaches zero, preemption is requested.

### Q8b: Is context switching in your kernel purely preemptive, cooperative, or hybrid?
**Answer:**  
It is a **hybrid cooperative / timer-monitored model**:  
- The **hardware timer interrupt (PIT IRQ 0)** acts as the real-time clock, continuously advancing ticks, decrementing remaining quantum budgets, and flagging quantum expiration.  
- The **context switch (`switch_context`)** is triggered cooperatively when tasks invoke `yield_now()`. This preserves stack integrity and avoids complex in-ISR asynchronous stack swapping, while guaranteeing deterministic dynamic time quantum enforcement.

### Q9: Why did you have to remap the 8259 PIC?
**Answer:**  
In IBM PC compatibles, the 8259 PIC defaults to mapping IRQs 0–7 to interrupt vectors 8–15. In protected and long mode, vectors 8–15 are reserved for CPU exceptions (e.g., vector 8 is Double Fault, 13 is General Protection Fault, 14 is Page Fault). If an IRQ 0 timer tick fires without remapping, the CPU treats it as a Double Fault exception! Remapping PIC1 to vector 32 (`0x20`) and PIC2 to vector 40 (`0x28`) prevents hardware IRQs from colliding with CPU exceptions.

### Q10: How does your Bump Allocator work?
**Answer:**  
The `BumpAllocator` manages a static 2 MiB heap buffer in kernel memory. It maintains a linear `next` pointer initialized to `heap_start`. When memory is requested with `allocate(size, align)`:
1. It calculates `aligned_next = (next + align - 1) & !(align - 1)`.
2. It checks whether `aligned_next + size <= heap_start + heap_size`.
3. If valid, it advances `self.next = aligned_next + size` and returns the pointer.
4. If it exceeds heap bounds, it returns `None` safely without panicking.
It provides $O(1)$ constant-time allocation and zero fragmentation during kernel boot.
