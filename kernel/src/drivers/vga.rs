//! VGA Text Mode Driver
//! Mayaank N - Micro-OS (25BTRCL148)

use crate::sync::SpinLock;
use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

pub struct VgaWriter {
    column_position: usize,
    color_code: ColorCode,
    buffer_base: usize,
}

impl VgaWriter {
    pub const fn new(buffer_base: usize) -> Self {
        Self {
            column_position: 0,
            color_code: ColorCode::new(Color::LightGreen, Color::Black),
            buffer_base,
        }
    }

    pub fn set_buffer_base(&mut self, base: usize) {
        self.buffer_base = base;
    }

    pub fn set_color(&mut self, color: ColorCode) {
        self.color_code = color;
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                self.write_char_at(row, col, ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.read_char_at(row, col);
                self.write_char_at(row - 1, col, character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.write_char_at(row, col, blank);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            self.clear_row(row);
        }
        self.column_position = 0;
    }

    #[inline]
    fn get_ptr(&self, row: usize, col: usize) -> *mut ScreenChar {
        let offset = (row * BUFFER_WIDTH + col) * 2;
        (self.buffer_base + offset) as *mut ScreenChar
    }

    fn write_char_at(&mut self, row: usize, col: usize, character: ScreenChar) {
        if self.buffer_base != 0 {
            unsafe {
                core::ptr::write_volatile(self.get_ptr(row, col), character);
            }
        }
    }

    fn read_char_at(&self, row: usize, col: usize) -> ScreenChar {
        if self.buffer_base != 0 {
            unsafe { core::ptr::read_volatile(self.get_ptr(row, col)) }
        } else {
            ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            }
        }
    }
}

impl fmt::Write for VgaWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

pub static VGA_WRITER: SpinLock<VgaWriter> = SpinLock::new(VgaWriter::new(0));

#[doc(hidden)]
pub fn _vga_print(args: fmt::Arguments) {
    use core::fmt::Write;
    VGA_WRITER.lock_irq_disabled().write_fmt(args).unwrap();
}
