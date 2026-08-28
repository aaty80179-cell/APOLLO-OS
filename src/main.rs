#![no_std]
#![no_main]
#![allow(dead_code)]

//! APOLLO OS Kernel - Main Entry Point
//! Byzantine-Resilient, Formally-Verified Microkernel
//!
//! Security Model:
//! - Capability-based access control
//! - Memory isolation via paging
//! - Information flow control
//! - Formal verification (Isabelle/HOL)

extern crate alloc;

use core::fmt::Write;
use core::panic::PanicInfo;

pub mod boot;
pub mod core;
pub mod memory;
pub mod security;
pub mod ipc;
pub mod scheduler;
pub mod capabilities;
pub mod crypto;

/// Kernel version
pub const KERNEL_VERSION: &str = "0.1.0-alpha";
pub const KERNEL_NAME: &str = "APOLLO OS";

/// Kernel entry point
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Initialize bootloader-provided data
    unsafe {
        boot::BOOTLOADER_INFO = Some(core::mem::zeroed());
    }

    // Initialize kernel
    kernel_init();

    // Should never return
    loop {
        core::hint::spin_loop();
    }
}

/// Main kernel initialization
fn kernel_init() {
    // ============================================
    // PHASE 1: Early Boot (Pre-Memory Management)
    // ============================================
    println!("\n[APOLLO] {} v{}", KERNEL_NAME, KERNEL_VERSION);
    println!("[APOLLO] Starting kernel initialization...");

    // Initialize CPU features
    core::cpu::init_cpu();
    println!("[APOLLO] CPU initialized");

    // ============================================
    // PHASE 2: Memory Initialization
    // ============================================
    memory::init_physical_memory();
    println!("[APOLLO] Physical memory initialized");

    memory::init_virtual_memory();
    println!("[APOLLO] Virtual memory initialized");

    // ============================================
    // PHASE 3: Kernel Data Structures
    // ============================================
    capabilities::init_capability_database();
    println!("[APOLLO] Capability system initialized");

    security::init_security_framework();
    println!("[APOLLO] Security framework initialized");

    // ============================================
    // PHASE 4: Process Management
    // ============================================
    scheduler::init_scheduler();
    println!("[APOLLO] Scheduler initialized");

    // ============================================
    // PHASE 5: IPC & Communication
    // ============================================
    ipc::init_message_bus();
    println!("[APOLLO] Message bus initialized");

    // ============================================
    // PHASE 6: Ready
    // ============================================
    println!("[APOLLO] Kernel ready");
    println!("[APOLLO] Entering main loop\n");
}

/// Panic handler - prints panic info and halts
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("\n[PANIC] {}", info);
    
    // Attempt to log panic to kernel log
    if let Some(location) = info.location() {
        println!("[PANIC] Location: {}:{}", location.file(), location.line());
    }
    
    println!("[PANIC] System halted\n");
    
    // Halt CPU
    loop {
        unsafe { core::arch::x86_64::hlt() };
    }
}

/// Simple println macro for kernel logging
#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}", format_args!($($arg)*));
        $crate::print!("\n");
    };
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        use core::fmt::Write;
        let _ = write!($crate::core::serial::SERIAL, "{}", format_args!($($arg)*));
    };
}
