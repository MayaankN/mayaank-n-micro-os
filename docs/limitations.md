# Architectural Limitations & Future Engineering Work

**Project Name:** Mayaank N – Micro-OS  
**Student Name:** Mayaank N  
**USN:** 25BTRCL148  
**Specialization:** Scheduler Engineering – Round Robin with Dynamic Quantum  

---

## 1. Current Architectural Limitations

### 1.1 Memory Allocator Reclamation
- **Limitation:** The current memory subsystem uses a `BumpAllocator`. While $O(1)$ fast and fragmentation-free, it advances a linear pointer and cannot reclaim or re-use individually deallocated objects (`kfree`).
- **Impact:** Memory allocations are monotonic. If tasks were frequently spawned and destroyed dynamically, the 2 MiB heap buffer would eventually be exhausted.

### 1.2 Privilege Level Isolation (Ring 0 vs. Ring 3)
- **Limitation:** All tasks in the microkernel currently execute in CPU Ring 0 (kernel mode) and share the higher-half virtual address space.
- **Impact:** Tasks are not isolated by hardware page table protection (`CR3`), meaning a buggy task could theoretically corrupt kernel structures or other task stacks. True user-space Ring 3 execution requires TSS (Task State Segment) configuration, `sysret`/`syscall` or `iretq` privilege drops, and per-process page tables.

### 1.3 Uniprocessor Execution (Single-Core)
- **Limitation:** Execution is currently constrained to the Bootstrap Processor (BSP) core in QEMU.
- **Impact:** Symmetric Multiprocessing (SMP) across multiple cores using ACPI MADT and LAPIC inter-processor interrupts (IPI) is not yet active.

### 1.4 Fixed Task Capacity
- **Limitation:** The scheduler maintains a statically allocated array of `MAX_TASKS = 16` task control blocks and a 32-element ring buffer ready queue.
- **Impact:** The system cannot exceed 16 concurrent tasks simultaneously without recompilation.

---

## 2. Future Engineering Roadmap

1. **Buddy / Slab Allocator Integration:** Replace the basic bump allocator with a two-level Buddy Allocator for 4 KiB physical page frames combined with a Slab Allocator for kernel object caches (`kfree` support).
2. **Ring 3 User Mode & Virtual Memory Isolation:** Implement user-space tasks running at Ring 3 with private page directories and a software syscall gate (`int 0x80` or `syscall`).
3. **Multi-core SMP Scheduler:** Implement per-core runqueues with work-stealing algorithms and APIC timer interrupts.
4. **VFS & Storage Driver:** Add a minimal virtual file system (VFS) with an in-memory Initrd / Tarfs or virtio-blk driver for persistent file I/O.
