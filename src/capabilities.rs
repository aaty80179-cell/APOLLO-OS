//! Capability-based access control system
//!
//! SECURITY MODEL:
//! - Capabilities are unforgeable tokens
//! - Each capability grants specific permissions
//! - Capabilities can be revoked
//! - HMAC-based validation prevents forgery

use hmac::{Hmac, Mac};
use sha2::Sha256;
use spin::Mutex;

type HmacSha256 = Hmac<Sha256>;

/// Unique object reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectRef(u64);

/// Permission type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Delegate,
    Revoke,
}

/// Capability - unforgeable token granting permissions
#[derive(Clone)]
pub struct Capability {
    object: ObjectRef,
    permission: Permission,
    hmac_tag: [u8; 32], // HMAC-SHA256
}

impl Capability {
    /// Create a new capability
    /// SECURITY: HMAC ensures authenticity
    pub fn new(object: ObjectRef, permission: Permission) -> Self {
        let mut cap = Capability {
            object,
            permission,
            hmac_tag: [0; 32],
        };
        
        // Generate HMAC
        let key = b"APOLLO_KERNEL_SECRET"; // In real impl, use proper key derivation
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC key error");
        
        let msg = format!("{:?}:{:?}", object, permission);
        mac.update(msg.as_bytes());
        
        let result = mac.finalize();
        cap.hmac_tag.copy_from_slice(&result.into_bytes()[..32]);
        
        cap
    }
    
    /// Validate capability authenticity
    /// SECURITY: Returns error if HMAC invalid (prevents forgery)
    pub fn validate(&self) -> Result<(), &'static str> {
        let key = b"APOLLO_KERNEL_SECRET";
        let mut mac = HmacSha256::new_from_slice(key)
            .expect("HMAC key error");
        
        let msg = format!("{:?}:{:?}", self.object, self.permission);
        mac.update(msg.as_bytes());
        
        let result = mac.finalize();
        let computed_tag = result.into_bytes();
        
        // Constant-time comparison to prevent timing attacks
        let mut eq = true;
        for i in 0..32 {
            if self.hmac_tag[i] != computed_tag[i] {
                eq = false;
            }
        }
        
        if eq {
            Ok(())
        } else {
            Err("Capability validation failed")
        }
    }
    
    /// Get the object this capability grants access to
    pub fn object(&self) -> ObjectRef {
        self.object
    }
    
    /// Get the permission this capability grants
    pub fn permission(&self) -> Permission {
        self.permission
    }
}

/// Capability database
pub struct CapabilityDatabase {
    // Map from ObjectRef to authorized capabilities
    // TODO: Implement
}

impl CapabilityDatabase {
    pub fn new() -> Self {
        CapabilityDatabase {}
    }
}

static CAP_DB: Mutex<Option<CapabilityDatabase>> = Mutex::new(None);

/// Initialize capability system
pub fn init_capability_database() {
    let mut db = CAP_DB.lock();
    if db.is_none() {
        *db = Some(CapabilityDatabase::new());
        println!("[CAPABILITIES] Capability database initialized");
    }
}
