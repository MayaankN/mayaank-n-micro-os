//! 16550 UART Serial Driver
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::drivers::port::{inb, outb};
use crate::sync::SpinLock;
use core::fmt;

pub const COM1_BASE: u16 = 0x3F8;

pub struct SerialPort {
    port: u16,
    initialized: bool,
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            initialized: false,
        }
    }

    /// Initializes the UART 16550 serial port for 115200 baud, 8N1.
    pub fn init(&mut self) {
        unsafe {
            // Disable all interrupts
            outb(self.port + 1, 0x00);
            // Enable DLAB (set baud rate divisor)
            outb(self.port + 3, 0x80);
            // Set divisor to 1 (115200 baud)
            outb(self.port + 0, 0x01);
            outb(self.port + 1, 0x00);
            // 8 bits, no parity, one stop bit (8N1)
            outb(self.port + 3, 0x03);
            // Enable FIFO, clear them, with 14-byte threshold
            outb(self.port + 2, 0xC7);
            // IRQs enabled, RTS/DSR set
            outb(self.port + 4, 0x0B);
        }
        self.initialized = true;
    }

    fn is_transmit_empty(&self) -> bool {
        unsafe { (inb(self.port + 5) & 0x20) != 0 }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if !self.initialized {
            self.init();
        }
        while !self.is_transmit_empty() {
            core::hint::spin_loop();
        }
        unsafe {
            outb(self.port, byte);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

pub static SERIAL1: SpinLock<SerialPort> = SpinLock::new(SerialPort::new(COM1_BASE));

#[doc(hidden)]
pub fn _serial_print(args: fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock_irq_disabled().write_fmt(args).unwrap();
}
