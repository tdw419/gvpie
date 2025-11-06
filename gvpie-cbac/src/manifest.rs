use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use crate::capability::{Capability, GpuOperation, Permission};
use anyhow::{Result, Context};

/// Delegation manifest loaded from TOML configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DelegationManifest {
    /// List of all delegations
    pub delegations: Vec<DelegationEntry>,

    /// Delegation rules by resource type
    #[serde(default)]
    pub delegation_rules: HashMap<String, HashMap<String, Vec<String>>>,

    /// System-wide permissions
    #[serde(default)]
    pub permissions: HashMap<String, Vec<String>>,
}

/// Single delegation entry in manifest
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DelegationEntry {
    /// Subject (process/daemon) receiving the capability
    pub subject: String,

    /// GPU operation being delegated
    pub operation: String,

    /// Permission level
    pub permission: String,

    /// Resource bounds
    #[serde(default)]
    pub bounds: ResourceBounds,

    /// Delegation metadata
    #[serde(default)]
    pub metadata: DelegationMetadata,
}

/// Resource bounds for a delegation
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ResourceBounds {
    /// Maximum VRAM in bytes
    #[serde(default)]
    pub max_vram_bytes: Option<usize>,

    /// Maximum compute units
    #[serde(default)]
    pub max_compute_units: Option<u32>,

    /// Maximum IPC messages per second
    #[serde(default)]
    pub max_ipc_messages: Option<u32>,
}

/// Delegation metadata
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DelegationMetadata {
    /// When this delegation was issued
    pub issued_at: String,

    /// When this delegation expires
    pub expires_at: String,

    /// Who issued this delegation
    pub issued_by: String,
}

impl Default for DelegationMetadata {
    fn default() -> Self {
        Self {
            issued_at: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::days(365)).to_rfc3339(),
            issued_by: "system".to_string(),
        }
    }
}

impl DelegationManifest {
    /// Load manifest from TOML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read delegation manifest")?;

        let manifest: DelegationManifest = toml::from_str(&content)
            .context("Failed to parse delegation manifest")?;

        Ok(manifest)
    }

    /// Save manifest to TOML file (atomic write)
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize delegation manifest")?;

        // Atomic write: write to temp file, then rename
        let temp_path = path.as_ref().with_extension("toml.tmp");
        fs::write(&temp_path, content)
            .context("Failed to write temporary manifest")?;

        fs::rename(&temp_path, path.as_ref())
            .context("Failed to rename temporary manifest")?;

        Ok(())
    }

    /// Find delegation for a specific subject and operation
    pub fn find_delegation(&self, subject: &str, operation: &str) -> Option<&DelegationEntry> {
        self.delegations.iter()
            .find(|d| d.subject == subject && d.operation == operation)
    }

    /// Convert delegation entry to capability token
    pub fn entry_to_capability(&self, entry: &DelegationEntry) -> Result<Capability> {
        // Parse operation
        let gpu_op = match entry.operation.as_str() {
            "RenderProgram" => GpuOperation::RenderProgram,
            "AllocateVRAM" => GpuOperation::AllocateVRAM,
            "ReadVRAM" => GpuOperation::ReadVRAM,
            "WriteVRAM" => GpuOperation::WriteVRAM,
            "ExecuteCompute" => GpuOperation::ExecuteCompute,
            "ReadMetrics" => GpuOperation::ReadMetrics,
            _ => anyhow::bail!("Unknown operation: {}", entry.operation),
        };

        // Parse permission
        let perm = match entry.permission.as_str() {
            "ReadOnly" => Permission::ReadOnly,
            "WriteOnly" => Permission::WriteOnly,
            "ReadWrite" => Permission::ReadWrite,
            "Execute" => Permission::Execute,
            _ => anyhow::bail!("Unknown permission: {}", entry.permission),
        };

        // Parse timestamps
        let issued_at = chrono::DateTime::parse_from_rfc3339(&entry.metadata.issued_at)
            .context("Invalid issued_at timestamp")?
            .with_timezone(&chrono::Utc);

        let expires_at = chrono::DateTime::parse_from_rfc3339(&entry.metadata.expires_at)
            .context("Invalid expires_at timestamp")?
            .with_timezone(&chrono::Utc);

        // Calculate duration
        let duration = expires_at.signed_duration_since(issued_at);

        Ok(Capability::with_expiry(
            entry.subject.clone(),
            gpu_op,
            perm,
            duration,
        ))
    }

    /// Load all delegations as capability tokens
    pub fn load_all_capabilities(&self) -> Result<Vec<Capability>> {
        self.delegations.iter()
            .map(|entry| self.entry_to_capability(entry))
            .collect()
    }

    /// Add a new delegation to the manifest
    pub fn add_delegation(&mut self, entry: DelegationEntry) {
        self.delegations.push(entry);
    }

    /// Remove a delegation by subject and operation
    pub fn remove_delegation(&mut self, subject: &str, operation: &str) -> bool {
        let original_len = self.delegations.len();
        self.delegations.retain(|d| !(d.subject == subject && d.operation == operation));
        self.delegations.len() < original_len
    }
}

/// Default manifest with zero-human-daemon permissions
impl Default for DelegationManifest {
    fn default() -> Self {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::days(365);

        let metadata = DelegationMetadata {
            issued_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
            issued_by: "system".to_string(),
        };

        Self {
            delegations: vec![
                DelegationEntry {
                    subject: "zero-human-daemon".to_string(),
                    operation: "RenderProgram".to_string(),
                    permission: "Execute".to_string(),
                    bounds: ResourceBounds {
                        max_vram_bytes: Some(16 * 1024 * 1024), // 16 MB
                        max_compute_units: Some(8),
                        max_ipc_messages: Some(1000),
                    },
                    metadata: metadata.clone(),
                },
                DelegationEntry {
                    subject: "zero-human-daemon".to_string(),
                    operation: "ReadMetrics".to_string(),
                    permission: "ReadOnly".to_string(),
                    bounds: ResourceBounds::default(),
                    metadata: metadata.clone(),
                },
            ],
            delegation_rules: HashMap::new(),
            permissions: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_manifest_load_save() {
        let manifest = DelegationManifest::default();

        let mut temp_file = NamedTempFile::new().unwrap();
        let toml_content = toml::to_string_pretty(&manifest).unwrap();
        temp_file.write_all(toml_content.as_bytes()).unwrap();

        let loaded = DelegationManifest::load(temp_file.path()).unwrap();
        assert_eq!(loaded.delegations.len(), 2);
    }

    #[test]
    fn test_find_delegation() {
        let manifest = DelegationManifest::default();

        let delegation = manifest.find_delegation("zero-human-daemon", "RenderProgram");
        assert!(delegation.is_some());

        let delegation = manifest.find_delegation("unknown", "RenderProgram");
        assert!(delegation.is_none());
    }

    #[test]
    fn test_entry_to_capability() {
        let manifest = DelegationManifest::default();
        let entry = &manifest.delegations[0];

        let capability = manifest.entry_to_capability(entry).unwrap();
        assert_eq!(capability.subject, "zero-human-daemon");
        assert!(capability.is_valid());
    }
}
