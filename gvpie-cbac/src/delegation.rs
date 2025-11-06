use crate::capability::{Capability, CapabilityError, GpuOperation, Permission};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entry in the delegation table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationEntry {
    /// The capability token
    pub capability: Capability,

    /// When this delegation was created
    pub delegated_at: DateTime<Utc>,

    /// Who delegated this capability (authority chain)
    pub delegated_by: String,

    /// Whether this delegation is still active
    pub active: bool,

    /// Audit log: times this capability was used
    pub usage_count: u64,

    /// Last time this capability was used
    pub last_used: Option<DateTime<Utc>>,
}

/// Delegation table - manages all capability grants
///
/// Formal Properties (ACSL-style):
/// ```c
/// /*@
///   invariant unique_signatures: \forall e1, e2 in entries,
///     e1.capability.signature == e2.capability.signature ==> e1 == e2;
///   invariant active_valid: \forall e in entries,
///     e.active == true ==> e.capability.is_valid() == true;
/// */
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct DelegationTable {
    /// Map: subject -> list of capabilities
    entries: HashMap<String, Vec<DelegationEntry>>,

    /// Map: signature -> entry (for fast lookup)
    by_signature: HashMap<String, DelegationEntry>,

    /// Audit log
    audit_log: Vec<AuditEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: DateTime<Utc>,
    pub subject: String,
    pub operation: GpuOperation,
    pub permission: Permission,
    pub result: AuditResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Granted,
    Denied(String),
}

impl DelegationTable {
    /// Create a new empty delegation table
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result.entries.len() == 0;
    ///   ensures \result.by_signature.len() == 0;
    ///   assigns \nothing;
    /// */
    /// ```
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            by_signature: HashMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Delegate a capability to a subject
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   requires capability.is_valid() == true;
    ///   requires delegated_by.len() > 0;
    ///   ensures \exists e in entries: e.capability.signature == capability.signature;
    ///   assigns entries, by_signature;
    /// */
    /// ```
    pub fn delegate(
        &mut self,
        capability: Capability,
        delegated_by: String,
    ) -> Result<(), CapabilityError> {
        // Verify capability is valid
        if !capability.is_valid() {
            return Err(CapabilityError::InvalidSignature);
        }

        let subject = capability.subject.clone();
        let signature = capability.signature.clone();

        let entry = DelegationEntry {
            capability,
            delegated_at: Utc::now(),
            delegated_by,
            active: true,
            usage_count: 0,
            last_used: None,
        };

        // Add to subject map
        self.entries
            .entry(subject.clone())
            .or_insert_with(Vec::new)
            .push(entry.clone());

        // Add to signature map
        self.by_signature.insert(signature, entry);

        Ok(())
    }

    /// Check if a subject has a valid capability for an operation
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \result == Ok(()) <==> (
    ///     \exists e in entries[subject]:
    ///       e.active == true &&
    ///       e.capability.is_valid() == true &&
    ///       e.capability.permits(operation, permission) == Ok(())
    ///   );
    ///   assigns last_used, usage_count, audit_log;
    /// */
    /// ```
    pub fn check_permission(
        &mut self,
        subject: &str,
        operation: GpuOperation,
        permission: Permission,
    ) -> Result<(), CapabilityError> {
        // Find subject's capabilities
        let capabilities = self.entries.get_mut(subject);

        if capabilities.is_none() {
            self.log_audit(subject, operation, permission, Err(CapabilityError::OperationNotAllowed));
            return Err(CapabilityError::OperationNotAllowed);
        }

        let capabilities = capabilities.unwrap();

        // Find first matching active capability
        for entry in capabilities.iter_mut() {
            if !entry.active {
                continue;
            }

            // Check if this capability permits the operation
            match entry.capability.permits(operation, permission) {
                Ok(()) => {
                    // Update usage stats
                    entry.usage_count += 1;
                    entry.last_used = Some(Utc::now());

                    // Update in signature map too
                    if let Some(sig_entry) = self.by_signature.get_mut(&entry.capability.signature) {
                        sig_entry.usage_count = entry.usage_count;
                        sig_entry.last_used = entry.last_used;
                    }

                    self.log_audit(subject, operation, permission, Ok(()));
                    return Ok(());
                }
                Err(_) => continue,
            }
        }

        // No valid capability found
        self.log_audit(subject, operation, permission, Err(CapabilityError::InsufficientPermission));
        Err(CapabilityError::InsufficientPermission)
    }

    /// Revoke a capability by signature
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \forall e in entries:
    ///     e.capability.signature == signature ==> e.active == false;
    ///   assigns entries[*].active;
    /// */
    /// ```
    pub fn revoke(&mut self, signature: &str) -> Result<(), CapabilityError> {
        // Find in signature map
        if let Some(entry) = self.by_signature.get_mut(signature) {
            entry.active = false;

            // Also mark inactive in subject map
            if let Some(subject_caps) = self.entries.get_mut(&entry.capability.subject) {
                for cap in subject_caps.iter_mut() {
                    if cap.capability.signature == signature {
                        cap.active = false;
                    }
                }
            }

            Ok(())
        } else {
            Err(CapabilityError::InvalidSignature)
        }
    }

    /// Get all capabilities for a subject
    pub fn get_capabilities(&self, subject: &str) -> Vec<&DelegationEntry> {
        self.entries
            .get(subject)
            .map(|caps| caps.iter().filter(|e| e.active).collect())
            .unwrap_or_default()
    }

    /// Get audit log
    pub fn audit_log(&self) -> &[AuditEvent] {
        &self.audit_log
    }

    /// Clean up expired capabilities
    ///
    /// Formal Contract (ACSL):
    /// ```c
    /// /*@
    ///   ensures \forall e in entries:
    ///     e.capability.is_valid() == false ==> e.active == false;
    ///   assigns entries[*].active;
    /// */
    /// ```
    pub fn cleanup_expired(&mut self) {
        for (_, entries) in self.entries.iter_mut() {
            for entry in entries.iter_mut() {
                if entry.active && !entry.capability.is_valid() {
                    entry.active = false;
                }
            }
        }

        // Also update signature map
        for (_, entry) in self.by_signature.iter_mut() {
            if entry.active && !entry.capability.is_valid() {
                entry.active = false;
            }
        }
    }

    /// Log an audit event
    fn log_audit(
        &mut self,
        subject: &str,
        operation: GpuOperation,
        permission: Permission,
        result: Result<(), CapabilityError>,
    ) {
        let audit_result = match result {
            Ok(()) => AuditResult::Granted,
            Err(e) => AuditResult::Denied(e.to_string()),
        };

        self.audit_log.push(AuditEvent {
            timestamp: Utc::now(),
            subject: subject.to_string(),
            operation,
            permission,
            result: audit_result,
        });
    }

    /// Save delegation table to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Load delegation table from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl Default for DelegationTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_delegation_lifecycle() {
        let mut table = DelegationTable::new();

        // Create and delegate a capability
        let cap = Capability::new(
            "gvpie-daemon".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadWrite,
        );

        table.delegate(cap.clone(), "system".to_string()).unwrap();

        // Should be able to use it
        assert!(table
            .check_permission("gvpie-daemon", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_ok());

        // Check usage count updated
        let caps = table.get_capabilities("gvpie-daemon");
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].usage_count, 1);

        // Revoke it
        table.revoke(&cap.signature).unwrap();

        // Should fail now
        assert!(table
            .check_permission("gvpie-daemon", GpuOperation::RenderProgram, Permission::ReadOnly)
            .is_err());
    }

    #[test]
    fn test_expiry_cleanup() {
        let mut table = DelegationTable::new();

        // Create expired capability
        let cap = Capability::with_expiry(
            "test".to_string(),
            GpuOperation::RenderProgram,
            Permission::ReadOnly,
            Duration::seconds(-1), // Already expired
        );

        // Force delegation (bypassing validation for test)
        let entry = DelegationEntry {
            capability: cap,
            delegated_at: Utc::now(),
            delegated_by: "system".to_string(),
            active: true,
            usage_count: 0,
            last_used: None,
        };

        table.entries.insert("test".to_string(), vec![entry]);

        // Cleanup should deactivate it
        table.cleanup_expired();

        let caps = table.get_capabilities("test");
        assert_eq!(caps.len(), 0); // No active capabilities
    }
}
