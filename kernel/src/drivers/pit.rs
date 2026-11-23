//! 8254 Programmable Interval Timer (PIT) Driver
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::drivers::port::outb;

pub const PIT_CHANNEL_0_DATA: u16 = 0x40;
pub const PIT_COMMAND_PORT: u16 = 0x43;
pub const PIT_BASE_FREQUENCY: u32 = 1193182;

/// Initializes the PIT to trigger IRQ0 at the requested frequency in Hz.
pub unsafe fn init_pit(frequency_hz: u32) {
    let divisor = (PIT_BASE_FREQUENCY / frequency_hz) as u16;

    // Channel 0, access mode: lobyte/hibyte, operating mode 2 (rate generator), binary mode
    outb(PIT_COMMAND_PORT, 0x34);

    // Write low byte then high byte of divisor
    outb(PIT_CHANNEL_0_DATA, (divisor & 0xFF) as u8);
    outb(PIT_CHANNEL_0_DATA, ((divisor >> 8) & 0xFF) as u8);
}
