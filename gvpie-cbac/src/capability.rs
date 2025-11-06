use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use thiserror::Error;

/// GPU operations that can be protected by capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuOperation {
    /// Render a pixel program to canvas
    RenderProgram,
    /// Allocate VRAM buffer
    AllocateVRAM,
    /// Read from VRAM
    ReadVRAM,
    /// Write to VRAM
    WriteVRAM,
    /// Execute compute shader
    ExecuteCompute,
    /// Access GPU metrics
    ReadMetrics,
}

/// Permission level for capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Execute,
}

/// Unforgeable capability token
///
/// Formal Properties (ACSL-style):
/// ```c
/// /*@
///   invariant valid_signature: signature == SHA256(subject || operation || issued_at || expires_at);
///   invariant temporal_validity: issued_at < expires_at;
///   invariant non_forgeable: \forall c1, c2: Capability,
///     (c1.signature == c2.signature) ==> (c1 == c2);
/// */
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Subject (process/daemon) that holds this capability
    pub subject: String,

    /// GPU operation this capability grants access to
    pub operation: GpuOperation,

    /// Permission level
    pub permission: Permission,

    /// When this capability was issued
    pub issued_at: DateTime<Utc>,

    /// When this capability expires (revocable)
    pub expires_at: DateTime<Utc>,

    /// Cryptographic signature (unforgeable)
    /// signature = SHA256(subject || operation || issued_at || expires_at || secret_key)
    pub signature: String,

    /// Optional metadata
    pub metadata: Option<String>,
}

#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("Capability has expired")]
    Expired,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Insufficient permission for operation")]
    InsufficientPermission,

    #[error("Operation not allowed by capability")]
    OperationNotAllowed,

    #[error("Capability revoked")]
    Revoked,
}

impl Capability {
    /// Create a new capability token
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   requires subject.len() > 0;
    ///   requires expires_at > issued_at;
    ///   ensures \result.is_valid() == true;
    ///   ensures \result.signature != "";
    ///   assigns \nothing;
    /// */
    /// ```
    pub fn new(subject: String, operation: GpuOperation, permission: Permission) -> Self {
        let issued_at = Utc::now();
        let expires_at = issued_at + Duration::hours(24); // Default 24h expiry

        let signature = Self::compute_signature(
            &subject,
            operation,
            permission,
            issued_at,
            expires_at,
        );

        Self {
            subject,
            operation,
            permission,
            issued_at,
            expires_at,
            signature,
            metadata: None,
        }
    }

    /// Create capability with custom expiry
    pub fn with_expiry(
        subject: String,
        operation: GpuOperation,
        permission: Permission,
        duration: Duration,
    ) -> Self {
        let issued_at = Utc::now();
        let expires_at = issued_at + duration;

        let signature = Self::compute_signature(
            &subject,
            operation,
            permission,
            issued_at,
            expires_at,
        );

        Self {
            subject,
            operation,
            permission,
            issued_at,
            expires_at,
            signature,
            metadata: None,
        }
    }

    /// Compute cryptographic signature
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result.len() == 64; // SHA256 hex digest
    ///   ensures deterministic:
    ///     compute_signature(s1, o1, p1, i1, e1) == compute_signature(s2, o2, p2, i2, e2)
    ///     <==> (s1 == s2 && o1 == o2 && p1 == p2 && i1 == i2 && e1 == e2);
    /// */
    /// ```
    fn compute_signature(
        subject: &str,
        operation: GpuOperation,
        permission: Permission,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Hash subject
        hasher.update(subject.as_bytes());

        // Hash operation
        hasher.update(&[operation as u8]);

        // Hash permission
        hasher.update(&[permission as u8]);

        // Hash timestamps
        hasher.update(issued_at.timestamp().to_le_bytes());
        hasher.update(expires_at.timestamp().to_le_bytes());

        // TODO: Add secret key from secure store
        // For now, use a deterministic constant (INSECURE - for demo only)
        hasher.update(b"GVPIE_SECRET_KEY_REPLACE_ME");

        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify this capability's signature
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result == true <==> signature == compute_signature(...);
    ///   assigns \nothing;
    /// */
    /// ```
    pub fn verify_signature(&self) -> bool {
        let expected = Self::compute_signature(
            &self.subject,
            self.operation,
            self.permission,
            self.issued_at,
            self.expires_at,
        );

        self.signature == expected
    }

    /// Check if capability is currently valid
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result == true <==> (
    ///     verify_signature() == true &&
    ///     Utc::now() >= issued_at &&
    ///     Utc::now() < expires_at
    ///   );
    ///   assigns \nothing;
    /// */
    /// ```
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();

        // Check signature
        if !self.verify_signature() {
            return false;
        }

        // Check temporal validity
        if now < self.issued_at || now >= self.expires_at {
            return false;
        }

        true
    }

    /// Check if this capability grants permission for an operation
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   requires is_valid() == true;
    ///   ensures \result == true <==> (
    ///     self.operation == requested_op &&
    ///     has_permission(self.permission, required_perm)
    ///   );
    /// */
    /// ```
    pub fn permits(
        &self,
        operation: GpuOperation,
        permission: Permission,
    ) -> Result<(), CapabilityError> {
        // Check validity first
        if !self.is_valid() {
            return Err(CapabilityError::Expired);
        }

        // Check operation match
        if self.operation != operation {
            return Err(CapabilityError::OperationNotAllowed);
        }

        // Check permission level
        if !self.has_permission(permission) {
            return Err(CapabilityError::InsufficientPermission);
        }

        Ok(())
    }

    /// Check if this capability's permission level satisfies the requirement
    fn has_permission(&self, required: Permission) -> bool {
        use Permission::*;

        match (self.permission, required) {
            // ReadWrite grants everything
            (ReadWrite, _) => true,

            // Exact match
            (ReadOnly, ReadOnly) => true,
            (WriteOnly, WriteOnly) => true,
            (Execute, Execute) => true,

            // Everything else is insufficient
            _ => false,
        }
    }

    /// Revoke this capability (by setting expiry to now)
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures expires_at == Utc::now();
    ///   ensures is_valid() == false;
    ///   assigns expires_at, signature;
    /// */
    /// ```
    pub fn revoke(&mut self) {
        self.expires_at = Utc::now();

        // Recompute signature with new expiry
        self.signature = Self::compute_signature(
            &self.subject,
            self.operation,
            self.permission,
            self.issued_at,
            self.expires_at,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_deterministic() {
        let cap1 = Capability::new(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );

        let cap2 = Capability {
            subject: cap1.subject.clone(),
            operation: cap1.operation,
            permission: cap1.permission,
            issued_at: cap1.issued_at,
            expires_at: cap1.expires_at,
            signature: String::new(),
            metadata: None,
        };

        let sig2 = Capability::compute_signature(
            &cap2.subject,
            cap2.operation,
            cap2.permission,
            cap2.issued_at,
            cap2.expires_at,
        );

        assert_eq!(cap1.signature, sig2);
    }

    #[test]
    fn test_permission_hierarchy() {
        let cap = Capability::new(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );

        assert!(cap.permits(GpuOperation::RenderProgram, Permission::ReadOnly).is_ok());
        assert!(cap.permits(GpuOperation::RenderProgram, Permission::WriteOnly).is_ok());
        assert!(cap.permits(GpuOperation::RenderProgram, Permission::ReadWrite).is_ok());

        // Wrong operation
        assert!(cap.permits(GpuOperation::AllocateVRAM, Permission::ReadOnly).is_err());
    }

    #[test]
    fn test_revocation() {
        let mut cap = Capability::new(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );

        assert!(cap.is_valid());

        cap.revoke();

        assert!(!cap.is_valid());
    }
}
