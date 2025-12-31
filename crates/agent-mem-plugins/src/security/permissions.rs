//! Permission checking for plugins

use agent_mem_plugin_sdk::Capability;
use anyhow::{anyhow, Result};

/// Permission checker
pub struct PermissionChecker {
    allowed_capabilities: Vec<Capability>,
}

impl PermissionChecker {
    pub fn new(allowed_capabilities: Vec<Capability>) -> Self {
        Self {
            allowed_capabilities,
        }
    }

    /// Check if a capability is allowed
    pub fn check(&self, required: &Capability) -> Result<()> {
        if self.allowed_capabilities.contains(required) {
            Ok(())
        } else {
            Err(anyhow!("Permission denied: {required:?} not allowed"))
        }
    }

    /// Check if all capabilities are allowed
    pub fn check_all(&self, required: &[Capability]) -> Result<()> {
        for cap in required {
            self.check(cap)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_check() {
        let checker =
            PermissionChecker::new(vec![Capability::MemoryAccess, Capability::LoggingAccess]);

        assert!(checker.check(&Capability::MemoryAccess).is_ok());
        assert!(checker.check(&Capability::LoggingAccess).is_ok());
        assert!(checker.check(&Capability::NetworkAccess).is_err());
    }

    #[test]
    fn test_permission_check_all() {
        let checker =
            PermissionChecker::new(vec![Capability::MemoryAccess, Capability::LoggingAccess]);

        assert!(checker
            .check_all(&[Capability::MemoryAccess, Capability::LoggingAccess])
            .is_ok());

        assert!(checker
            .check_all(&[Capability::MemoryAccess, Capability::NetworkAccess])
            .is_err());
    }
}
