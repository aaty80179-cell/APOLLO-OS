//! Memory management - Physical and virtual memory

pub mod allocator;
pub mod paging;
pub mod physical;

use crate::println;

/// Initialize physical memory management
pub fn init_physical_memory() {
    physical::init_physical_allocator();
}

/// Initialize virtual memory (paging)
pub fn init_virtual_memory() {
    paging::init_paging();
}
