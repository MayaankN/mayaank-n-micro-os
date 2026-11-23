//! Kernel Diagnostics and Telemetry
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::kprintln;
use crate::memory::KERNEL_ALLOCATOR;
use crate::scheduler::{SCHEDULER, MAX_TASKS};
use crate::task::TaskId;
use crate::timer::current_ticks;

/// Dumps a complete snapshot of kernel scheduler and task telemetry.
pub fn dump_scheduler_diagnostics() {
    let ticks = current_ticks();
    kprintln!("\n================== KERNEL SCHEDULER DIAGNOSTICS ==================");
    kprintln!("Timestamp: {} ticks | Specialization: Dynamic Quantum Round Robin", ticks);
    
    let (switches, rq_len) = {
        let sched = SCHEDULER.lock_irq_disabled();
        (sched.total_context_switches(), sched.ready_queue_len())
    };

    kprintln!("Total Context Switches: {}", switches);
    kprintln!("Ready Queue Depth:      {}", rq_len);
    kprintln!("------------------------------------------------------------------");
    kprintln!("{:<8} {:<18} {:<10} {:<10} {:<12} {:<10}", "TASK ID", "NAME", "STATE", "QUANTUM", "CPU TICKS", "SWITCHES");
    kprintln!("------------------------------------------------------------------");

    let sched = SCHEDULER.lock_irq_disabled();
    for i in 0..MAX_TASKS {
        if let Some((name, state, quantum, cpu_ticks, switches)) = sched.get_task_info(TaskId(i as u64)) {
            kprintln!(
                "{:<8} {:<18} {:<10} {:<10} {:<12} {:<10}",
                format_args!("Task-{}", i),
                name,
                state.as_str(),
                format_args!("{} ticks", quantum),
                cpu_ticks,
                switches
            );
        }
    }
    kprintln!("==================================================================\n");
}

/// Dumps memory allocator statistics.
pub fn dump_memory_diagnostics() {
    let alloc = KERNEL_ALLOCATOR.lock_irq_disabled();
    kprintln!("\n================== KERNEL MEMORY DIAGNOSTICS =====================");
    kprintln!("Allocator:          Linear Bump Allocator");
    kprintln!("Heap Capacity:      {} bytes ({} KiB)", alloc.total_capacity(), alloc.total_capacity() / 1024);
    kprintln!("Allocated Memory:   {} bytes ({} KiB)", alloc.used_bytes(), alloc.used_bytes() / 1024);
    kprintln!("Available Memory:   {} bytes ({} KiB)", alloc.remaining_bytes(), alloc.remaining_bytes() / 1024);
    kprintln!("Allocation Count:   {}", alloc.allocation_count());
    kprintln!("==================================================================\n");
}
