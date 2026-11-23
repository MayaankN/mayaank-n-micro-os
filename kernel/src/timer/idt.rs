//! Interrupt Descriptor Table (IDT) and Exception Handling
//! Mayaank N - Micro-OS (25BTRCL148)

use core::cell::UnsafeCell;
use crate::drivers::PICS;
use crate::klog_trace;
use crate::scheduler::SCHEDULER;
use crate::timer::timer::increment_ticks;
use core::arch::{asm, naked_asm};

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IdtEntry {
    pointer_low: u16,
    gdt_selector: u16,
    options: u16,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            pointer_low: 0,
            gdt_selector: 0,
            options: 0,
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }

    pub fn set_handler(&mut self, handler: usize, cs: u16) {
        self.pointer_low = handler as u16;
        self.gdt_selector = cs;    // Use active 64-bit kernel code segment selector
        self.options = 0x8E00;     // Present, 64-bit Interrupt Gate, DPL=0
        self.pointer_middle = (handler >> 16) as u16;
        self.pointer_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
pub struct Idtr {
    limit: u16,
    base: u64,
}

pub const IDT_ENTRIES: usize = 256;

struct SyncIdt(UnsafeCell<[IdtEntry; IDT_ENTRIES]>);
unsafe impl Sync for SyncIdt {}

static IDT_TABLE: SyncIdt = SyncIdt(UnsafeCell::new([IdtEntry::empty(); IDT_ENTRIES]));

/// Reads current active code segment selector (CS).
#[inline]
fn read_cs() -> u16 {
    let cs: u16;
    unsafe {
        asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack, preserves_flags));
    }
    cs
}

/// Initializes the IDT and loads it into the CPU with lidt.
pub fn init_idt() {
    let cs = read_cs();
    unsafe {
        let idt = &mut *IDT_TABLE.0.get();

        // Exception 0: Divide Error
        idt[0].set_handler(divide_error_handler as *const () as usize, cs);
        // Exception 13: General Protection Fault
        idt[13].set_handler(general_protection_fault_handler as *const () as usize, cs);
        // Exception 14: Page Fault
        idt[14].set_handler(page_fault_handler as *const () as usize, cs);

        // Vector 32 (IRQ 0): PIT Timer Tick
        idt[32].set_handler(timer_interrupt_entry as *const () as usize, cs);

        let idtr = Idtr {
            limit: (core::mem::size_of::<[IdtEntry; IDT_ENTRIES]>() - 1) as u16,
            base: idt.as_ptr() as u64,
        };

        asm!("lidt [{}]", in(reg) &idtr, options(readonly, nostack, preserves_flags));
    }
}

/// Enables CPU interrupts.
#[inline]
pub fn enable_interrupts() {
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

/// Disables CPU interrupts.
#[inline]
pub fn disable_interrupts() {
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

#[unsafe(naked)]
unsafe extern "C" fn timer_interrupt_entry() {
    naked_asm!(
        // Save scratch caller-saved registers
        "push rax",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",

        // Call Rust timer handler
        "call {timer_handler}",

        // Restore registers
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rax",

        // Return from interrupt
        "iretq",
        timer_handler = sym timer_handler_inner
    );
}

#[no_mangle]
extern "C" fn timer_handler_inner() {
    let tick = increment_ticks();

    unsafe {
        PICS.lock().notify_end_of_interrupt(32);
    }

    let reschedule = {
        let mut sched = SCHEDULER.lock();
        sched.on_timer_tick(tick)
    };

    if reschedule && tick % 50 == 0 {
        klog_trace!("Timer tick {:>5} | Preemption check triggered", tick);
    }
}

#[unsafe(naked)]
unsafe extern "C" fn divide_error_handler() {
    naked_asm!(
        "cli",
        "2: hlt",
        "jmp 2b"
    );
}

#[unsafe(naked)]
unsafe extern "C" fn general_protection_fault_handler() {
    naked_asm!(
        "cli",
        "2: hlt",
        "jmp 2b"
    );
}

#[unsafe(naked)]
unsafe extern "C" fn page_fault_handler() {
    naked_asm!(
        "cli",
        "2: hlt",
        "jmp 2b"
    );
}
