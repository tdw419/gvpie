use crate::capability::{Capability, CapabilityError, GpuOperation, Permission};
use crate::delegation::DelegationTable;
use crate::manifest::DelegationManifest;
use std::sync::{Arc, Mutex};
use std::path::Path;

/// Capability verifier - enforces CBAC policies
///
/// Formal Properties (ACSL-style):
/// ```c
/// /*@
///   invariant delegation_integrity: \forall t in delegation_tables,
///     t satisfies DelegationTable invariants;
///   invariant verification_deterministic: \forall cap, op, perm,
///     verify(cap, op, perm) returns same result for same inputs;
/// */
/// ```
pub struct CapabilityVerifier {
    /// Delegation table (shared, thread-safe)
    delegation_table: Arc<Mutex<DelegationTable>>,

    /// Whether to enforce CBAC (can be disabled for testing)
    enforce: bool,
}

impl CapabilityVerifier {
    /// Create a new verifier
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result.enforce == true;
    ///   ensures \result.delegation_table is initialized;
    ///   assigns \nothing;
    /// */
    /// ```
    pub fn new() -> Self {
        Self {
            delegation_table: Arc::new(Mutex::new(DelegationTable::new())),
            enforce: true,
        }
    }

    /// Create verifier with existing delegation table
    pub fn with_table(table: DelegationTable) -> Self {
        Self {
            delegation_table: Arc::new(Mutex::new(table)),
            enforce: true,
        }
    }

    /// Create verifier from a delegation manifest file
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   requires path points to valid TOML file;
    ///   ensures \result.Ok() ==> (
    ///     \forall entry in manifest.delegations:
    ///       \exists cap in delegation_table:
    ///         cap.signature == entry_to_capability(entry).signature
    ///   );
    ///   assigns delegation_table;
    /// */
    /// ```
    pub fn from_manifest<P: AsRef<Path>>(path: P) -> Result<Self, CapabilityError> {
        // Load manifest from TOML file
        let manifest = DelegationManifest::load(path)
            .map_err(|_| CapabilityError::InvalidSignature)?;

        // Create empty delegation table
        let mut table = DelegationTable::new();

        // Convert all manifest entries to capabilities and delegate them
        for entry in &manifest.delegations {
            let capability = manifest.entry_to_capability(entry)
                .map_err(|_| CapabilityError::InvalidSignature)?;

            // Delegate with issuer from metadata
            table.delegate(capability, entry.metadata.issued_by.clone())
                .map_err(|_| CapabilityError::InvalidSignature)?;
        }

        Ok(Self::with_table(table))
    }

    /// Disable enforcement (TESTING ONLY)
    pub fn disable_enforcement(&mut self) {
        self.enforce = false;
    }

    /// Verify a capability grants permission for an operation
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   requires enforce == true;
    ///   ensures \result == Ok(()) <==> (
    ///     delegation_table.check_permission(
    ///       capability.subject, operation, permission
    ///     ) == Ok(())
    ///   );
    ///   assigns delegation_table.last_used, delegation_table.usage_count;
    /// */
    /// ```
    pub fn verify(
        &self,
        subject: &str,
        operation: GpuOperation,
        permission: Permission,
    ) -> Result<(), CapabilityError> {
        // If enforcement disabled, allow everything
        if !self.enforce {
            return Ok(());
        }

        // Lock delegation table
        let mut table = self.delegation_table
            .lock()
            .map_err(|_| CapabilityError::InvalidSignature)?;

        // Check permission
        table.check_permission(subject, operation, permission)
    }

    /// Verify with explicit capability token (for stateless verification)
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result == Ok(()) <==> (
    ///     capability.is_valid() == true &&
    ///     capability.permits(operation, permission) == Ok(())
    ///   );
    ///   assigns \nothing;
    /// */
    /// ```
    pub fn verify_capability(
        &self,
        capability: &Capability,
        operation: GpuOperation,
        permission: Permission,
    ) -> Result<(), CapabilityError> {
        // If enforcement disabled, allow everything
        if !self.enforce {
            return Ok(());
        }

        // Check capability validity and permissions
        capability.permits(operation, permission)
    }

    /// Delegate a new capability
    pub fn delegate(
        &self,
        capability: Capability,
        delegated_by: String,
    ) -> Result<(), CapabilityError> {
        let mut table = self.delegation_table
            .lock()
            .map_err(|_| CapabilityError::InvalidSignature)?;

        table.delegate(capability, delegated_by)
    }

    /// Revoke a capability
    pub fn revoke(&self, signature: &str) -> Result<(), CapabilityError> {
        let mut table = self.delegation_table
            .lock()
            .map_err(|_| CapabilityError::InvalidSignature)?;

        table.revoke(signature)
    }

    /// Get delegation table snapshot
    pub fn get_table_snapshot(&self) -> Result<DelegationTable, CapabilityError> {
        let table = self.delegation_table
            .lock()
            .map_err(|_| CapabilityError::InvalidSignature)?;

        // Clone the entire table
        Ok(DelegationTable::from_json(&table.to_json().unwrap()).unwrap())
    }

    /// Cleanup expired capabilities
    pub fn cleanup_expired(&self) -> Result<(), CapabilityError> {
        let mut table = self.delegation_table
            .lock()
            .map_err(|_| CapabilityError::InvalidSignature)?;

        table.cleanup_expired();
        Ok(())
    }

    /// Get shared reference to delegation table (for advanced use)
    pub fn get_table(&self) -> Arc<Mutex<DelegationTable>> {
        Arc::clone(&self.delegation_table)
    }
}

impl Default for CapabilityVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for verifying capabilities in function calls
///
/// Example:
/// ```rust
/// verify_cap!(verifier, "gvpie-daemon", GpuOperation::RenderProgram, Permission::Execute);
/// ```
#[macro_export]
macro_rules! verify_cap {
    ($verifier:expr, $subject:expr, $op:expr, $perm:expr) => {
        $verifier.verify($subject, $op, $perm)?
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_basic() {
        let verifier = CapabilityVerifier::new();

        // Create and delegate capability
        let cap = Capability::new(
            "test-subject".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );

        verifier.delegate(cap.clone(), "system".to_string()).unwrap();

        // Should pass verification
        assert!(verifier
            .verify("test-subject", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_ok());

        // Wrong subject should fail
        assert!(verifier
            .verify("wrong-subject", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_err());

        // Wrong operation should fail
        assert!(verifier
            .verify("test-subject", GpuOperation::AllocateVRAM, Permission::ReadOnly)
            .is_err());
    }

    #[test]
    fn test_stateless_verification() {
        let verifier = CapabilityVerifier::new();

        let cap = Capability::new(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::Execute,
        );

        // Stateless verification (doesn't need delegation table)
        assert!(verifier
            .verify_capability(&cap, GpuOperation::RenderProgram, Permission::Execute)
            .is_ok());

        // Wrong permission
        assert!(verifier
            .verify_capability(&cap, GpuOperation::RenderProgram, Permission::ReadWrite)
            .is_err());
    }

    #[test]
    fn test_revocation() {
        let verifier = CapabilityVerifier::new();

        let cap = Capability::new(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );

        let signature = cap.signature.clone();

        verifier.delegate(cap, "system".to_string()).unwrap();

        // Should work before revocation
        assert!(verifier
            .verify("test", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_ok());

        // Revoke
        verifier.revoke(&signature).unwrap();

        // Should fail after revocation
        assert!(verifier
            .verify("test", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_err());
    }

    #[test]
    fn test_enforcement_toggle() {
        let mut verifier = CapabilityVerifier::new();

        // No capabilities delegated, should fail
        assert!(verifier
            .verify("test", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_err());

        // Disable enforcement
        verifier.disable_enforcement();

        // Should pass now (enforcement disabled)
        assert!(verifier
            .verify("test", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_ok());
    }

    #[test]
    fn test_from_manifest() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use crate::manifest::DelegationManifest;

        // Create a test manifest
        let manifest = DelegationManifest::default();

        // Save to temp file
        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = toml::to_string_pretty(&manifest).unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        // Load verifier from manifest
        let verifier = CapabilityVerifier::from_manifest(temp_file.path()).unwrap();

        // Should be able to verify zero-human-daemon capabilities
        assert!(verifier
            .verify("zero-human-daemon", GpuOperation::RenderProgram, Permission::Execute)
            .is_ok());

        assert!(verifier
            .verify("zero-human-daemon", GpuOperation::ReadMetrics, Permission::ReadOnly)
            .is_ok());

        // Should fail for unauthorized subject
        assert!(verifier
            .verify("unauthorized", GpuOperation::RenderProgram, Permission::Execute)
            .is_err());
    }
}
