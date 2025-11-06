"""
CBAC Manifest Loader - Load capabilities from TOML manifest files

This module provides Python utilities to load and parse CBAC delegation
manifests (TOML format) and initialize the CBAC client accordingly.
"""

import tomllib  # Python 3.11+, use 'tomli' package for older versions
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime, timezone

# Handle imports for both module and script usage
try:
    from .cbac_client import (
        CBACClient,
        GpuOperation,
        Permission,
        CapabilityError
    )
except ImportError:
    from cbac_client import (
        CBACClient,
        GpuOperation,
        Permission,
        CapabilityError
    )


class ManifestLoader:
    """
    Loads CBAC delegation manifests from TOML files

    The manifest format matches the Rust implementation in gvpie-cbac/src/manifest.rs
    """

    def __init__(self, manifest_path: Path):
        """
        Initialize manifest loader

        Args:
            manifest_path: Path to TOML manifest file
        """
        self.manifest_path = manifest_path
        self.manifest_data: Optional[Dict[str, Any]] = None

    def load(self) -> Dict[str, Any]:
        """
        Load manifest from TOML file

        Returns:
            Parsed manifest data

        Raises:
            FileNotFoundError: If manifest file doesn't exist
            ValueError: If manifest is invalid
        """
        if not self.manifest_path.exists():
            raise FileNotFoundError(f"Manifest not found: {self.manifest_path}")

        with open(self.manifest_path, 'rb') as f:
            self.manifest_data = tomllib.load(f)

        self._validate_manifest()
        return self.manifest_data

    def _validate_manifest(self):
        """Validate manifest structure"""
        if not self.manifest_data:
            raise ValueError("Manifest is empty")

        if "delegations" not in self.manifest_data:
            raise ValueError("Manifest missing 'delegations' section")

        # Check each delegation has required fields
        for i, delegation in enumerate(self.manifest_data["delegations"]):
            required = ["subject", "operation", "permission", "metadata"]
            for field in required:
                if field not in delegation:
                    raise ValueError(f"Delegation {i} missing required field: {field}")

    def get_delegations_for_subject(self, subject: str) -> List[Dict[str, Any]]:
        """
        Get all delegations for a specific subject

        Args:
            subject: Subject name (e.g., "zero-human-daemon")

        Returns:
            List of delegation entries for the subject
        """
        if not self.manifest_data:
            self.load()

        return [
            d for d in self.manifest_data["delegations"]
            if d["subject"] == subject
        ]

    def initialize_cbac_client(
        self,
        subject: str,
        client: Optional[CBACClient] = None
    ) -> CBACClient:
        """
        Initialize CBAC client with capabilities from manifest

        Args:
            subject: Subject to load capabilities for
            client: Existing client to update (creates new if None)

        Returns:
            CBAC client with loaded capabilities
        """
        if client is None:
            client = CBACClient()

        delegations = self.get_delegations_for_subject(subject)

        if not delegations:
            print(f"⚠️  No delegations found for subject: {subject}")
            return client

        # Load each delegation as a capability
        for delegation in delegations:
            try:
                operation = self._parse_operation(delegation["operation"])
                permission = self._parse_permission(delegation["permission"])

                # Calculate duration from metadata
                metadata = delegation.get("metadata", {})
                issued_at = datetime.fromisoformat(metadata.get("issued_at", datetime.now(timezone.utc).isoformat()))
                expires_at = datetime.fromisoformat(metadata.get("expires_at", datetime.now(timezone.utc).isoformat()))

                duration_hours = int((expires_at - issued_at).total_seconds() / 3600)

                # Request capability
                cap = client.request_capability(
                    subject=subject,
                    operation=operation,
                    permission=permission,
                    duration_hours=duration_hours
                )

                print(f"✅ Loaded capability: {subject} → {operation.value} ({permission.value})")

            except Exception as e:
                print(f"⚠️  Failed to load delegation: {e}")
                continue

        return client

    def _parse_operation(self, operation_str: str) -> GpuOperation:
        """Parse operation string to enum"""
        mapping = {
            "RenderProgram": GpuOperation.RENDER_PROGRAM,
            "AllocateVRAM": GpuOperation.ALLOCATE_VRAM,
            "ReadVRAM": GpuOperation.READ_VRAM,
            "WriteVRAM": GpuOperation.WRITE_VRAM,
            "ExecuteCompute": GpuOperation.EXECUTE_COMPUTE,
            "ReadMetrics": GpuOperation.READ_METRICS,
        }

        if operation_str not in mapping:
            raise ValueError(f"Unknown operation: {operation_str}")

        return mapping[operation_str]

    def _parse_permission(self, permission_str: str) -> Permission:
        """Parse permission string to enum"""
        mapping = {
            "ReadOnly": Permission.READ_ONLY,
            "WriteOnly": Permission.WRITE_ONLY,
            "ReadWrite": Permission.READ_WRITE,
            "Execute": Permission.EXECUTE,
        }

        if permission_str not in mapping:
            raise ValueError(f"Unknown permission: {permission_str}")

        return mapping[permission_str]

    def get_resource_bounds(self, subject: str, operation: str) -> Optional[Dict[str, int]]:
        """
        Get resource bounds for a subject's operation

        Args:
            subject: Subject name
            operation: Operation name

        Returns:
            Resource bounds dict or None if not found
        """
        delegations = self.get_delegations_for_subject(subject)

        for delegation in delegations:
            if delegation["operation"] == operation:
                return delegation.get("bounds", {})

        return None

    def get_delegation_rules(self, resource_type: str) -> Dict[str, List[str]]:
        """
        Get delegation rules for a resource type

        Args:
            resource_type: Resource type (e.g., "compute", "vram", "ipc")

        Returns:
            Mapping of subject -> list of allowed resources
        """
        if not self.manifest_data:
            self.load()

        delegation_rules = self.manifest_data.get("delegation_rules", {})
        return delegation_rules.get(resource_type, {})


def load_manifest_for_daemon(
    subject: str = "zero-human-daemon",
    manifest_path: Optional[Path] = None
) -> CBACClient:
    """
    Convenience function to load CBAC client from manifest

    Args:
        subject: Subject name to load capabilities for
        manifest_path: Path to manifest (uses default if None)

    Returns:
        Initialized CBAC client
    """
    if manifest_path is None:
        # Use default manifest location
        manifest_path = Path(__file__).parent.parent.parent / "cbac_manifests" / "default_capabilities.toml"

    loader = ManifestLoader(manifest_path)
    loader.load()

    print(f"📋 Loading CBAC manifest from: {manifest_path}")

    client = loader.initialize_cbac_client(subject)

    print(f"✅ CBAC client initialized for {subject}")

    return client


if __name__ == "__main__":
    # Demo: Load manifest and initialize client
    import sys

    subject = sys.argv[1] if len(sys.argv) > 1 else "zero-human-daemon"

    try:
        client = load_manifest_for_daemon(subject)

        # Show loaded capabilities
        caps = client.get_capabilities(subject)
        print(f"\n📊 Loaded {len(caps)} capabilities:")
        for cap in caps:
            print(f"  - {cap['operation']} ({cap['permission']})")

        # Test a permission check
        print(f"\n🔐 Testing permission check:")
        allowed = client.check_permission(
            subject,
            GpuOperation.RENDER_PROGRAM,
            Permission.EXECUTE
        )
        print(f"  RenderProgram/Execute: {'✅ ALLOWED' if allowed else '❌ DENIED'}")

    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)
