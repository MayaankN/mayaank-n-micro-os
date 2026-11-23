//! 8259 Dual PIC (Programmable Interrupt Controller) Driver
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::drivers::port::{inb, io_wait, outb};
use crate::sync::SpinLock;

pub const PIC1_CMD: u16 = 0x20;
pub const PIC1_DATA: u16 = 0x21;
pub const PIC2_CMD: u16 = 0xA0;
pub const PIC2_DATA: u16 = 0xA1;

pub const PIC_EOI: u8 = 0x20;

// Re-map offsets
pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = 40;

pub struct ChainedPics {
    offset1: u8,
    offset2: u8,
}

impl ChainedPics {
    pub const fn new(offset1: u8, offset2: u8) -> Self {
        Self { offset1, offset2 }
    }

    /// Remaps the PICs to the specified vector offsets.
    pub unsafe fn initialize(&self) {
        // Save interrupt masks
        let mask1 = inb(PIC1_DATA);
        let mask2 = inb(PIC2_DATA);

        // ICW1: Start initialization in cascade mode
        outb(PIC1_CMD, 0x11);
        io_wait();
        outb(PIC2_CMD, 0x11);
        io_wait();

        // ICW2: Vector offsets
        outb(PIC1_DATA, self.offset1);
        io_wait();
        outb(PIC2_DATA, self.offset2);
        io_wait();

        // ICW3: Cascading setup
        outb(PIC1_DATA, 4); // Tell Master that Slave is at IRQ2
        io_wait();
        outb(PIC2_DATA, 2); // Tell Slave its cascade identity
        io_wait();

        // ICW4: 8086/88 mode
        outb(PIC1_DATA, 0x01);
        io_wait();
        outb(PIC2_DATA, 0x01);
        io_wait();

        // Restore masks (or enable IRQ0 timer)
        outb(PIC1_DATA, mask1);
        outb(PIC2_DATA, mask2);
    }

    /// Sets the interrupt mask for PIC1 and PIC2.
    pub unsafe fn set_mask(&self, mask1: u8, mask2: u8) {
        outb(PIC1_DATA, mask1);
        outb(PIC2_DATA, mask2);
    }

    /// Sends End-of-Interrupt (EOI) for the specified interrupt vector.
    pub unsafe fn notify_end_of_interrupt(&self, interrupt_id: u8) {
        if interrupt_id >= self.offset2 && interrupt_id < self.offset2 + 8 {
            outb(PIC2_CMD, PIC_EOI);
            outb(PIC1_CMD, PIC_EOI);
        } else if interrupt_id >= self.offset1 && interrupt_id < self.offset1 + 8 {
            outb(PIC1_CMD, PIC_EOI);
        }
    }
}

pub static PICS: SpinLock<ChainedPics> = SpinLock::new(ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET));
