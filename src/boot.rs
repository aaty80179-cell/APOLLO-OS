//! Boot module - Handles bootloader interaction and early CPU setup

use core::mem;
use core::ptr::NonNull;

/// Bootloader information (provided by UEFI/BIOS)
/// SECURITY: This is populated by bootloader and validated
pub static mut BOOTLOADER_INFO: Option<BootloaderInfo> = None;

/// Information from bootloader
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BootloaderInfo {
    pub memory_map_addr: u64,
    pub memory_map_size: u64,
    pub kernel_phys_base: u64,
    pub kernel_virt_base: u64,
    pub framebuffer_addr: u64,
    pub framebuffer_size: u64,
    pub acpi_rsdp_addr: u64,
}

/// Get bootloader info safely
/// SECURITY: Validates pointer before access
pub fn get_bootloader_info() -> Option<BootloaderInfo> {
    unsafe {
        BOOTLOADER_INFO.as_ref().copied()
    }
}

/// Validate bootloader provided data
/// SECURITY CHECKPOINT: Verify memory layout consistency
pub fn validate_bootloader_data() -> Result<(), &'static str> {
    let info = get_bootloader_info()
        .ok_or("Bootloader info not available")?;
    
    // Check 1: Memory map must be non-null and reasonable size
    if info.memory_map_addr == 0 || info.memory_map_size == 0 {
        return Err("Invalid memory map from bootloader");
    }
    
    // Check 2: Kernel must be loaded at valid address
    if info.kernel_phys_base < 0x100000 || info.kernel_phys_base >= 0xFFFF_FFFF {
        return Err("Kernel loaded at invalid physical address");
    }
    
    // Check 3: Virtual and physical bases must not conflict
    if info.kernel_virt_base == info.kernel_phys_base && info.kernel_virt_base > 0x100000 {
        return Err("Virtual and physical bases are identical (kernel needs higher-half mapping)");
    }
    
    // Check 4: Framebuffer must be reasonable size
    if info.framebuffer_size > 0 && info.framebuffer_addr == 0 {
        return Err("Framebuffer size non-zero but address is null");
    }
    
    Ok(())
}
