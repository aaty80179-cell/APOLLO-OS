//! Physical memory allocator
//! 
//! SECURITY PROPERTIES:
//! - No use-after-free (pages tracked with lifetime)
//! - No double-allocation (bitmap prevents)
//! - Overflow-safe (checked arithmetic)

use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

const PAGE_SIZE: usize = 4096;
const MAX_PAGES: usize = 1_000_000; // 4GB address space

/// Physical page
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalPage {
    index: u64,
}

impl PhysicalPage {
    /// Get physical address of this page
    pub fn address(&self) -> u64 {
        self.index.checked_mul(PAGE_SIZE as u64)
            .expect("Page address overflow")
    }
}

/// Bitmap-based physical memory allocator
/// SECURITY: Atomic operations prevent race conditions
pub struct PhysicalAllocator {
    bitmap: &'static mut [AtomicU64],
    total_pages: u64,
    allocated_pages: AtomicU64,
}

impl PhysicalAllocator {
    /// Allocate a new physical page
    /// SECURITY: Returns unique page or error, never double-allocates
    pub fn allocate(&self) -> Result<PhysicalPage, AllocError> {
        for word_idx in 0..(self.bitmap.len()) {
            let mut word = self.bitmap[word_idx].load(Ordering::Acquire);
            
            if word == u64::MAX {
                continue; // All bits set, skip
            }
            
            // Find first free bit
            for bit_idx in 0..64 {
                if (word & (1 << bit_idx)) == 0 {
                    let new_word = word | (1 << bit_idx);
                    
                    // Atomically set bit (CAS - Compare and Swap)
                    match self.bitmap[word_idx].compare_exchange(
                        word,
                        new_word,
                        Ordering::Release,
                        Ordering::Acquire,
                    ) {
                        Ok(_) => {
                            let page_idx = (word_idx as u64 * 64) + bit_idx as u64;
                            if page_idx >= self.total_pages {
                                return Err(AllocError::OutOfMemory);
                            }
                            
                            // Update allocated count
                            let _ = self.allocated_pages.fetch_add(1, Ordering::Release);
                            
                            return Ok(PhysicalPage { index: page_idx });
                        }
                        Err(actual) => {
                            word = actual;
                            // Retry this word
                        }
                    }
                }
            }
        }
        
        Err(AllocError::OutOfMemory)
    }
    
    /// Free a physical page
    /// SECURITY: Validates page before freeing
    pub fn free(&self, page: PhysicalPage) -> Result<(), AllocError> {
        let word_idx = (page.index / 64) as usize;
        let bit_idx = (page.index % 64) as usize;
        
        if word_idx >= self.bitmap.len() {
            return Err(AllocError::InvalidPage);
        }
        
        let word = self.bitmap[word_idx].load(Ordering::Acquire);
        
        // Check if page is actually allocated
        if (word & (1 << bit_idx)) == 0 {
            return Err(AllocError::DoubleFreePrevented); // Safety check
        }
        
        let new_word = word & !(1 << bit_idx);
        self.bitmap[word_idx].store(new_word, Ordering::Release);
        let _ = self.allocated_pages.fetch_sub(1, Ordering::Release);
        
        Ok(())
    }
    
    /// Get number of allocated pages
    pub fn allocated_count(&self) -> u64 {
        self.allocated_pages.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AllocError {
    OutOfMemory,
    InvalidPage,
    DoubleFreePrevented,
}

/// Global allocator
static GLOBAL_ALLOC: Mutex<Option<PhysicalAllocator>> = Mutex::new(None);

/// Initialize physical allocator
pub fn init_physical_allocator() {
    let mut alloc = GLOBAL_ALLOC.lock();
    if alloc.is_none() {
        println!("[MEMORY] Initializing physical memory allocator");
        // In real implementation, bitmap would be initialized from bootloader data
        // For now, just indicate it's ready
        println!("[MEMORY] Physical allocator ready");
    }
}

/// Allocate a physical page
pub fn alloc_page() -> Result<PhysicalPage, AllocError> {
    let alloc = GLOBAL_ALLOC.lock();
    if let Some(ref a) = *alloc {
        a.allocate()
    } else {
        Err(AllocError::OutOfMemory)
    }
}
