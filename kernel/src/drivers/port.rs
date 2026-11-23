//! x86 Port I/O Driver
//! Mayaank N - Micro-OS (25BTRCL148)

use core::arch::asm;

/// Reads an 8-bit byte from the specified I/O port.
#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    value
}

/// Writes an 8-bit byte to the specified I/O port.
#[inline]
pub unsafe fn outb(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

/// Reads a 16-bit word from the specified I/O port.
#[inline]
pub unsafe fn inw(port: u16) -> u16 {
    let value: u16;
    asm!("in ax, dx", in("dx") port, out("ax") value, options(nomem, nostack, preserves_flags));
    value
}

/// Writes a 16-bit word to the specified I/O port.
#[inline]
pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

/// Small delay port write to allow slow legacy hardware devices to settle.
#[inline]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
