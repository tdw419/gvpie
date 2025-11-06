"""
CBAC Client - Python interface to Capability-Based Access Control

This module provides Python bindings to the Rust CBAC implementation via JSON IPC.
"""

import json
import subprocess
from typing import Optional, Dict, Any
from pathlib import Path
from enum import Enum

class GpuOperation(Enum):
    """GPU operations that require capabilities"""
    RENDER_PROGRAM = "RenderProgram"
    ALLOCATE_VRAM = "AllocateVRAM"
    READ_VRAM = "ReadVRAM"
    WRITE_VRAM = "WriteVRAM"
    EXECUTE_COMPUTE = "ExecuteCompute"
    READ_METRICS = "ReadMetrics"

class Permission(Enum):
    """Permission levels"""
    READ_ONLY = "ReadOnly"
    WRITE_ONLY = "WriteOnly"
    READ_WRITE = "ReadWrite"
    EXECUTE = "Execute"

class CapabilityError(Exception):
    """CBAC-related errors"""
    pass

class CBACClient:
    """
    Client for interacting with CBAC verifier

    In production, this would communicate with the Rust verifier via:
    - Unix domain socket
    - Shared memory
    - Or embedded Rust library (PyO3)

    For now, uses JSON file-based IPC.
    """

    def __init__(self, delegation_table_path: Optional[Path] = None):
        """
        Initialize CBAC client

        Args:
            delegation_table_path: Path to delegation table JSON file
        """
        self.delegation_table_path = delegation_table_path or Path("/tmp/gvpie/delegation_table.json")
        self.delegation_table_path.parent.mkdir(parents=True, exist_ok=True)

        # In-memory cache of capabilities
        self._capabilities: Dict[str, Dict[str, Any]] = {}

        # Load existing table if present
        if self.delegation_table_path.exists():
            self._load_table()

    def _load_table(self):
        """Load delegation table from disk"""
        try:
            with open(self.delegation_table_path) as f:
                data = json.load(f)
                self._capabilities = data.get("entries", {})
        except Exception as e:
            print(f"Warning: Failed to load delegation table: {e}")

    def _save_table(self):
        """Save delegation table to disk"""
        try:
            data = {"entries": self._capabilities}
            with open(self.delegation_table_path, 'w') as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            print(f"Warning: Failed to save delegation table: {e}")

    def request_capability(
        self,
        subject: str,
        operation: GpuOperation,
        permission: Permission,
        duration_hours: int = 24
    ) -> Dict[str, Any]:
        """
        Request a new capability token

        Args:
            subject: Process/daemon requesting the capability
            operation: GPU operation to authorize
            permission: Permission level
            duration_hours: How long the capability should be valid

        Returns:
            Capability token dictionary

        Raises:
            CapabilityError: If capability cannot be granted
        """
        # In production, this would call into Rust verifier
        # For now, generate a simple token

        capability = {
            "subject": subject,
            "operation": operation.value,
            "permission": permission.value,
            "issued_at": self._get_timestamp(),
            "expires_at": self._get_timestamp(hours_offset=duration_hours),
            "signature": self._compute_signature(subject, operation, permission),
        }

        # Store in delegation table
        if subject not in self._capabilities:
            self._capabilities[subject] = []

        self._capabilities[subject].append(capability)
        self._save_table()

        return capability

    def check_permission(
        self,
        subject: str,
        operation: GpuOperation,
        permission: Permission
    ) -> bool:
        """
        Check if subject has permission for operation

        Args:
            subject: Process/daemon to check
            operation: GPU operation
            permission: Required permission level

        Returns:
            True if permission granted, False otherwise
        """
        # Load fresh table
        self._load_table()

        # Find capabilities for subject
        subject_caps = self._capabilities.get(subject, [])

        for cap in subject_caps:
            # Check if capability matches
            if (cap.get("operation") == operation.value and
                self._permission_satisfies(cap.get("permission"), permission.value) and
                self._is_valid(cap)):
                return True

        return False

    def verify_and_execute(
        self,
        subject: str,
        operation: GpuOperation,
        permission: Permission,
        action: callable
    ) -> Any:
        """
        Verify permission then execute action

        Args:
            subject: Process/daemon
            operation: GPU operation
            permission: Required permission
            action: Function to execute if permission granted

        Returns:
            Result of action()

        Raises:
            CapabilityError: If permission denied
        """
        if not self.check_permission(subject, operation, permission):
            raise CapabilityError(
                f"Permission denied: {subject} cannot perform {operation.value} "
                f"with {permission.value} permission"
            )

        return action()

    def revoke_capability(self, subject: str, signature: str):
        """Revoke a capability by signature"""
        if subject in self._capabilities:
            self._capabilities[subject] = [
                cap for cap in self._capabilities[subject]
                if cap.get("signature") != signature
            ]
            self._save_table()

    def get_capabilities(self, subject: str) -> list:
        """Get all capabilities for a subject"""
        self._load_table()
        return self._capabilities.get(subject, [])

    def _get_timestamp(self, hours_offset: int = 0) -> str:
        """Get ISO timestamp"""
        from datetime import datetime, timedelta, timezone
        dt = datetime.now(timezone.utc) + timedelta(hours=hours_offset)
        return dt.isoformat()

    def _compute_signature(self, subject: str, operation: GpuOperation, permission: Permission) -> str:
        """Compute simple signature (INSECURE - for demo only)"""
        import hashlib
        data = f"{subject}:{operation.value}:{permission.value}".encode()
        return hashlib.sha256(data).hexdigest()[:16]

    def _permission_satisfies(self, granted: str, required: str) -> bool:
        """Check if granted permission satisfies required permission"""
        if granted == "ReadWrite":
            return True
        if granted == required:
            return True
        return False

    def _is_valid(self, cap: Dict[str, Any]) -> bool:
        """Check if capability is currently valid"""
        from datetime import datetime, timezone

        try:
            expires = datetime.fromisoformat(cap.get("expires_at", ""))
            now = datetime.now(timezone.utc)
            return now < expires
        except:
            return False


# Default global CBAC client
_cbac_client: Optional[CBACClient] = None

def get_cbac_client() -> CBACClient:
    """Get global CBAC client instance"""
    global _cbac_client
    if _cbac_client is None:
        _cbac_client = CBACClient()
    return _cbac_client


def init_daemon_capabilities(subject: str = "zero-human-daemon"):
    """
    Initialize default capabilities for the daemon

    This should be called at daemon startup to grant necessary permissions.
    """
    client = get_cbac_client()

    # Grant daemon permission to render programs
    client.request_capability(
        subject=subject,
        operation=GpuOperation.RENDER_PROGRAM,
        permission=Permission.EXECUTE,
        duration_hours=24 * 365  # 1 year
    )

    # Grant daemon permission to read metrics
    client.request_capability(
        subject=subject,
        operation=GpuOperation.READ_METRICS,
        permission=Permission.READ_ONLY,
        duration_hours=24 * 365
    )

    print(f"✅ Initialized CBAC capabilities for {subject}")


if __name__ == "__main__":
    # Demo usage
    client = CBACClient()

    # Request capability
    cap = client.request_capability(
        "gvpie-daemon",
        GpuOperation.RENDER_PROGRAM,
        Permission.EXECUTE
    )
    print(f"Granted capability: {cap}")

    # Check permission
    allowed = client.check_permission(
        "gvpie-daemon",
        GpuOperation.RENDER_PROGRAM,
        Permission.EXECUTE
    )
    print(f"Permission check: {allowed}")

    # Verify and execute
    def render_action():
        print("Rendering program...")
        return "success"

    result = client.verify_and_execute(
        "gvpie-daemon",
        GpuOperation.RENDER_PROGRAM,
        Permission.EXECUTE,
        render_action
    )
    print(f"Execution result: {result}")
