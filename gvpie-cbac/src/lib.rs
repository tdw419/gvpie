/// GVPIE CBAC - Capability-Based Access Control for GPU Resources
///
/// This module implements unforgeable capability tokens for GPU resource access.
/// Provides formal guarantees:
/// - Non-forgeable tokens (cryptographic signatures)
/// - Revocable capabilities (timestamp-based expiry)
/// - Fine-grained permissions (opcode-level granularity)
/// - Audit trail (all capability checks logged)

pub mod capability;
pub mod delegation;
pub mod verifier;

pub use capability::{Capability, CapabilityError, GpuOperation, Permission};
pub use delegation::{DelegationTable, DelegationEntry};
pub use verifier::CapabilityVerifier;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(
            "gvpie-daemon".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );
        assert_eq!(cap.subject, "gvpie-daemon");
        assert!(cap.is_valid());
    }

    #[test]
    fn test_capability_expiry() {
        let mut cap = Capability::new(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadOnly,
        );

        // Set expiry in the past
        cap.expires_at = chrono::Utc::now() - chrono::Duration::seconds(1);
        assert!(!cap.is_valid());
    }
}
