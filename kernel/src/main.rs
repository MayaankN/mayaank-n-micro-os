//! Mayaank N - Micro-OS (CA6 Engineering Project)
//! Student Name: Mayaank N
//! USN: 25BTRCL148
//! Specialization: Scheduler Engineering (Round Robin with Dynamic Quantum)

#![no_std]
#![no_main]

pub mod drivers;
pub mod kernel;
pub mod memory;
pub mod scheduler;
pub mod sync;
pub mod task;
pub mod timer;

use core::panic::PanicInfo;
use drivers::serial::SERIAL1;
use drivers::vga::{Color, ColorCode, VGA_WRITER};
use drivers::{init_pit, PICS};
use kernel::diagnostics::{dump_memory_diagnostics, dump_scheduler_diagnostics};
use kernel::limine::HHDM_REQUEST;
use memory::{init_heap, kmalloc};
use scheduler::{switch_context, yield_now, SCHEDULER};
use task::{TaskId, DEFAULT_STACK_SIZE};
use timer::{current_ticks, enable_interrupts, init_idt};

/// Kernel Entry Point invoked by Limine Bootloader
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Initialize Serial Communication (COM1) for headless diagnostics
    SERIAL1.lock().init();

    // 2. Initialize VGA Text Console / Framebuffer
    init_console();

    // 3. Display Project Header & Identification
    print_banner();

    // 4. Initialize Kernel Memory Heap and Bump Allocator
    klog_info!("Initializing kernel bump memory allocator...");
    init_heap();
    dump_memory_diagnostics();

    // 5. Initialize Dual 8259 PIC Interrupt Controller
    klog_info!("Initializing 8259 PIC (Remapping IRQs 0-15 to vectors 32-47)...");
    unsafe {
        PICS.lock().initialize();
        // Unmask IRQ0 (Timer)
        PICS.lock().set_mask(0xFE, 0xFF);
    }

    // 6. Initialize Interrupt Descriptor Table (IDT)
    klog_info!("Loading Interrupt Descriptor Table (IDT)...");
    init_idt();

    // 7. Initialize 8254 Programmable Interval Timer (100 Hz = 10ms ticks)
    klog_info!("Configuring 8254 PIT at 100 Hz...");
    unsafe {
        init_pit(100);
    }

    // 8. Allocate Stacks and Spawn Controlled Tasks
    klog_info!("Spawning controlled kernel tasks into Ready Queue...");
    spawn_kernel_tasks();

    dump_scheduler_diagnostics();

    // 9. Enable CPU Interrupts
    klog_info!("Enabling CPU hardware interrupts (sti)...");
    enable_interrupts();

    // 10. Bootstrap Scheduler: Switch from boot thread to first ready task
    klog_sched!("Dispatching first task from Ready Queue...");
    let switch_data = {
        let mut sched = SCHEDULER.lock_irq_disabled();
        sched.schedule_next(current_ticks())
    };

    if let Some((_old_ctx, new_ctx, _prev, next_id, quantum)) = switch_data {
        klog_sched!("Bootstrap context switch into {} (Quantum: {} ticks)", next_id, quantum);
        let mut dummy_context = task::TaskContext::empty();
        unsafe {
            switch_context(&mut dummy_context, new_ctx);
        }
    }

    // Fallback CPU halt loop if scheduler ever returns
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Spawns the required controlled kernel tasks.
fn spawn_kernel_tasks() {
    let mut sched = SCHEDULER.lock_irq_disabled();
    let tick = current_ticks();

    // Task 1: Compute-bound task (exercises full quantum, triggering quantum expansion)
    let stack1 = kmalloc(DEFAULT_STACK_SIZE, 16).expect("Failed to allocate Task 1 stack");
    sched.spawn_task(
        TaskId(1),
        "Compute-Worker",
        task_compute_worker as *const () as usize,
        1,
        stack1 as usize,
        tick,
    );

    // Task 2: Telemetry task (periodic system inspector and trace generator)
    let stack2 = kmalloc(DEFAULT_STACK_SIZE, 16).expect("Failed to allocate Task 2 stack");
    sched.spawn_task(
        TaskId(2),
        "Telemetry-Worker",
        task_telemetry_worker as *const () as usize,
        2,
        stack2 as usize,
        tick,
    );

    // Task 3: Burst-Yield task (interactive short-burst worker triggering downward quantum adaptation)
    let stack3 = kmalloc(DEFAULT_STACK_SIZE, 16).expect("Failed to allocate Task 3 stack");
    sched.spawn_task(
        TaskId(3),
        "Burst-Worker",
        task_burst_worker as *const () as usize,
        1,
        stack3 as usize,
        tick,
    );
}

/// Task 1: Compute-Worker (CPU bound, tests time slice execution and quantum expansion)
extern "C" fn task_compute_worker() -> ! {
    klog_info!("Task 1 [Compute-Worker] started. Running iterative compute workloads.");
    let mut counter: u64 = 0;
    loop {
        counter = counter.wrapping_add(1);
        klog_trace!("Task 1 [Compute-Worker] cycle #{}: Computing math workload", counter);
        
        for _ in 0..5_000 {
            core::hint::spin_loop();
        }
        
        // Cooperatively yield to cycle round-robin
        yield_now();
    }
}

/// Task 2: Telemetry-Worker (Periodic system diagnostics reporter)
extern "C" fn task_telemetry_worker() -> ! {
    klog_info!("Task 2 [Telemetry-Worker] started. Monitoring system telemetry.");
    let mut reports = 0;
    loop {
        reports += 1;
        klog_info!("Task 2 [Telemetry-Worker] generating periodic report #{}", reports);
        dump_scheduler_diagnostics();
        dump_memory_diagnostics();

        for _ in 0..5_000 {
            core::hint::spin_loop();
        }
        yield_now();
    }
}

/// Task 3: Burst-Worker (Short bursts, exercises dynamic quantum adaptation)
extern "C" fn task_burst_worker() -> ! {
    klog_info!("Task 3 [Burst-Worker] started. Simulating interactive I/O bursts.");
    let mut iteration = 0;
    loop {
        iteration += 1;
        klog_trace!("Task 3 [Burst-Worker] cycle #{}: Interactive short burst complete", iteration);
        for _ in 0..1_000 {
            core::hint::spin_loop();
        }
        yield_now();
    }
}

/// Initializes VGA buffer base address from HHDM or standard identity map.
fn init_console() {
    let hhdm = unsafe { HHDM_REQUEST.response.as_ref() };
    let vga_base = if let Some(resp) = hhdm {
        (resp.offset + 0xb8000) as usize
    } else {
        0xffffffff800b8000
    };
    VGA_WRITER.lock().set_buffer_base(vga_base);
    VGA_WRITER.lock().clear_screen();
}

/// Prints formatted boot banner with student identification.
fn print_banner() {
    VGA_WRITER.lock().set_color(ColorCode::new(Color::Cyan, Color::Black));
    kprintln!("================================================================================");
    kprintln!("          MAYAANK N - MICRO-OS: BARE-METAL RUST OPERATING SYSTEM               ");
    kprintln!("================================================================================");
    VGA_WRITER.lock().set_color(ColorCode::new(Color::Yellow, Color::Black));
    kprintln!(" Project Owner:  Mayaank N");
    kprintln!(" USN:            25BTRCL148");
    kprintln!(" Course:         Operating Systems (CA6 Engineering Project)");
    kprintln!(" Architecture:   x86_64 Bare-Metal (no_std, no_main)");
    kprintln!(" Bootloader:     Limine Boot Protocol v8");
    kprintln!(" Specialization: Scheduler Engineering - Round Robin with Dynamic Quantum");
    VGA_WRITER.lock().set_color(ColorCode::new(Color::LightGreen, Color::Black));
    kprintln!("================================================================================");
    kprintln!();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    VGA_WRITER.lock().set_color(ColorCode::new(Color::White, Color::Red));
    kprintln!("\n!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    kprintln!("                           KERNEL PANIC OCCURRED                        ");
    kprintln!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
    kprintln!("{}", info);
    kprintln!("Halting CPU execution.");
    loop {
        unsafe {
            core::arch::asm!("cli; hlt");
        }
    }
}
