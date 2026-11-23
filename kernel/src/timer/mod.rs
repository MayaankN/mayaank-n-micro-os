//! Hardware Timer and IDT Subsystem
//! Mayaank N - Micro-OS (25BTRCL148)

pub mod idt;
pub mod timer;

pub use idt::{disable_interrupts, enable_interrupts, init_idt};
pub use timer::{current_ticks, increment_ticks};
