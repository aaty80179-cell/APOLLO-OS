//! Serial port output for debugging

use core::fmt;
use spin::Mutex;

/// COM1 port address
const COM1: u16 = 0x3F8;

/// Thread-safe serial output
pub static SERIAL: Mutex<Serial> = Mutex::new(Serial {});

pub struct Serial {}

impl Serial {
    /// Initialize COM1 serial port
    pub fn init() {
        // Set baud rate divisor
        unsafe {
            core::arch::x86_64::__outb(COM1 + 1, 0x00);
            core::arch::x86_64::__outb(COM1 + 3, 0x80);
            core::arch::x86_64::__outb(COM1 + 0, 0x03); // 115200 baud
            core::arch::x86_64::__outb(COM1 + 1, 0x00);
            core::arch::x86_64::__outb(COM1 + 3, 0x03);
            core::arch::x86_64::__outb(COM1 + 2, 0xC7);
            core::arch::x86_64::__outb(COM1 + 4, 0x0B);
        }
    }
    
    /// Write a single byte
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            core::arch::x86_64::__outb(COM1, byte);
        }
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}
