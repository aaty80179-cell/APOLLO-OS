//! Core CPU functionality
//! Initializes CPU features needed for kernel operation

use core::arch::x86_64::{
    _rdmsr, _wrmsr, _xsave, _xrstor,
    CpuidResult, __cpuid, __cpuid_count,
};

/// CPU feature flags
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    pub has_msr: bool,
    pub has_pae: bool,
    pub has_pse: bool,
    pub has_pge: bool,
    pub has_fxsr: bool,
    pub has_sse: bool,
    pub has_xsave: bool,
    pub has_nx: bool,
    pub has_syscall: bool,
    pub has_rdtscp: bool,
}

impl CpuFeatures {
    /// Detect CPU capabilities
    /// SECURITY: Check required features for secure kernel operation
    pub fn detect() -> Self {
        let cpuid_1 = unsafe { __cpuid(1) };
        let cpuid_ext = unsafe { __cpuid(0x80000001) };
        
        CpuFeatures {
            has_msr: (cpuid_1.edx & (1 << 5)) != 0,
            has_pae: (cpuid_1.edx & (1 << 6)) != 0,
            has_pse: (cpuid_1.edx & (1 << 3)) != 0,
            has_pge: (cpuid_1.edx & (1 << 13)) != 0,
            has_fxsr: (cpuid_1.edx & (1 << 24)) != 0,
            has_sse: (cpuid_1.edx & (1 << 25)) != 0,
            has_xsave: (cpuid_1.ecx & (1 << 26)) != 0,
            has_nx: (cpuid_ext.edx & (1 << 20)) != 0,
            has_syscall: (cpuid_ext.edx & (1 << 11)) != 0,
            has_rdtscp: (cpuid_ext.edx & (1 << 27)) != 0,
        }
    }
    
    /// Verify required features are present
    /// SECURITY: Panic if critical features missing
    pub fn verify_required() -> Result<(), &'static str> {
        let features = Self::detect();
        
        // These are absolutely required
        if !features.has_pae {
            return Err("CPU missing PAE (Physical Address Extension)");
        }
        if !features.has_pge {
            return Err("CPU missing PGE (Page Global Enable)");
        }
        if !features.has_nx {
            return Err("CPU missing NX bit (No-Execute)");
        }
        if !features.has_msr {
            return Err("CPU missing MSR support");
        }
        
        Ok(())
    }
}

/// Initialize CPU for kernel operation
pub fn init_cpu() {
    // Verify required features exist
    if let Err(e) = CpuFeatures::verify_required() {
        panic!("CPU verification failed: {}", e);
    }
    
    // Detect optional features
    let features = CpuFeatures::detect();
    println!("[CPU] Features: PAE={}, NX={}, SYSCALL={}",
             features.has_pae, features.has_nx, features.has_syscall);
    
    // Enable critical MSRs
    unsafe {
        // Enable NX bit via EFER MSR
        if features.has_nx {
            let mut efer = _rdmsr(0xC0000080); // IA32_EFER
            efer |= 0x800; // Set NX Enable bit
            _wrmsr(0xC0000080, efer);
        }
        
        // Enable SMEP (Supervisor Mode Execution Prevention) if available
        // This prevents kernel from accidentally executing user code
        let cr4_smep = 0x100000; // SMEP bit
        let mut cr4: u64;
        core::arch::x86_64::__asm!(
            "mov rax, cr4",
            out("rax") cr4,
            options(nostack, preserves_flags)
        );
        cr4 |= cr4_smep;
        core::arch::x86_64::__asm!(
            "mov cr4, rax",
            in("rax") cr4,
            options(nostack, preserves_flags)
        );
    }
    
    println!("[CPU] Initialization complete");
}
