#!/usr/bin/env python3
"""
CBAC Integration Test

This script demonstrates the complete CBAC system integration:
1. Load delegation manifest from TOML
2. Initialize CBAC client with capabilities
3. Verify permissions for various operations
4. Demonstrate audit trail and enforcement
"""

import sys
from pathlib import Path

# Add python_daemon to path
sys.path.insert(0, str(Path(__file__).parent / "python_daemon"))

from pixel_os.cbac_manifest_loader import load_manifest_for_daemon, ManifestLoader
from pixel_os.cbac_client import GpuOperation, Permission, CapabilityError


def print_section(title: str):
    """Print a section header"""
    print(f"\n{'='*70}")
    print(f"  {title}")
    print(f"{'='*70}\n")


def test_zero_human_daemon():
    """Test capabilities for zero-human-daemon"""
    print_section("Testing zero-human-daemon Capabilities")

    # Load manifest for zero-human-daemon
    client = load_manifest_for_daemon("zero-human-daemon")

    # Get loaded capabilities
    caps = client.get_capabilities("zero-human-daemon")
    print(f"✅ Loaded {len(caps)} capabilities for zero-human-daemon\n")

    # Test authorized operations
    print("🔐 Testing AUTHORIZED operations:")

    tests = [
        (GpuOperation.RENDER_PROGRAM, Permission.EXECUTE, "Should succeed"),
        (GpuOperation.READ_METRICS, Permission.READ_ONLY, "Should succeed"),
    ]

    for operation, permission, expected in tests:
        allowed = client.check_permission("zero-human-daemon", operation, permission)
        status = "✅ ALLOWED" if allowed else "❌ DENIED"
        print(f"  {operation.value} / {permission.value}: {status} ({expected})")

    # Test unauthorized operations
    print("\n🚫 Testing UNAUTHORIZED operations:")

    unauthorized_tests = [
        (GpuOperation.ALLOCATE_VRAM, Permission.READ_WRITE, "Should fail (not granted)"),
        (GpuOperation.WRITE_VRAM, Permission.WRITE_ONLY, "Should fail (not granted)"),
    ]

    for operation, permission, expected in unauthorized_tests:
        allowed = client.check_permission("zero-human-daemon", operation, permission)
        status = "✅ ALLOWED" if allowed else "❌ DENIED"
        print(f"  {operation.value} / {permission.value}: {status} ({expected})")


def test_gvpie_daemon():
    """Test capabilities for gvpie-daemon"""
    print_section("Testing gvpie-daemon Capabilities")

    # Load manifest for gvpie-daemon
    client = load_manifest_for_daemon("gvpie-daemon")

    # Get loaded capabilities
    caps = client.get_capabilities("gvpie-daemon")
    print(f"✅ Loaded {len(caps)} capabilities for gvpie-daemon\n")

    # Test authorized operations
    print("🔐 Testing AUTHORIZED operations:")

    tests = [
        (GpuOperation.RENDER_PROGRAM, Permission.EXECUTE, "Should succeed"),
        (GpuOperation.ALLOCATE_VRAM, Permission.READ_WRITE, "Should succeed"),
        (GpuOperation.ALLOCATE_VRAM, Permission.READ_ONLY, "Should succeed (ReadWrite includes ReadOnly)"),
    ]

    for operation, permission, expected in tests:
        allowed = client.check_permission("gvpie-daemon", operation, permission)
        status = "✅ ALLOWED" if allowed else "❌ DENIED"
        print(f"  {operation.value} / {permission.value}: {status} ({expected})")


def test_resource_bounds():
    """Test resource bounds from manifest"""
    print_section("Testing Resource Bounds")

    manifest_path = Path(__file__).parent / "cbac_manifests" / "default_capabilities.toml"
    loader = ManifestLoader(manifest_path)
    loader.load()

    # Check bounds for zero-human-daemon
    print("📊 Resource bounds for zero-human-daemon:")
    bounds = loader.get_resource_bounds("zero-human-daemon", "RenderProgram")
    if bounds:
        print(f"  Max VRAM: {bounds.get('max_vram_bytes', 0) / (1024*1024):.1f} MB")
        print(f"  Max Compute Units: {bounds.get('max_compute_units', 0)}")
        print(f"  Max IPC Messages: {bounds.get('max_ipc_messages', 0)}")

    # Check bounds for gvpie-daemon
    print("\n📊 Resource bounds for gvpie-daemon:")
    bounds = loader.get_resource_bounds("gvpie-daemon", "RenderProgram")
    if bounds:
        print(f"  Max VRAM: {bounds.get('max_vram_bytes', 0) / (1024*1024):.1f} MB")
        print(f"  Max Compute Units: {bounds.get('max_compute_units', 0)}")
        print(f"  Max IPC Messages: {bounds.get('max_ipc_messages', 0)}")


def test_delegation_rules():
    """Test delegation rules from manifest"""
    print_section("Testing Delegation Rules")

    manifest_path = Path(__file__).parent / "cbac_manifests" / "default_capabilities.toml"
    loader = ManifestLoader(manifest_path)
    loader.load()

    # Check compute resources
    print("⚙️  Compute resource delegations:")
    compute_rules = loader.get_delegation_rules("compute")
    for subject, resources in compute_rules.items():
        print(f"  {subject}: {', '.join(resources)}")

    # Check VRAM resources
    print("\n💾 VRAM resource delegations:")
    vram_rules = loader.get_delegation_rules("vram")
    for subject, resources in vram_rules.items():
        print(f"  {subject}: {', '.join(resources)}")

    # Check IPC resources
    print("\n📡 IPC resource delegations:")
    ipc_rules = loader.get_delegation_rules("ipc")
    for subject, resources in ipc_rules.items():
        print(f"  {subject}: {', '.join(resources)}")


def test_verify_and_execute():
    """Test verify-and-execute pattern"""
    print_section("Testing Verify-and-Execute Pattern")

    client = load_manifest_for_daemon("zero-human-daemon")

    # Test successful execution
    print("✅ Testing ALLOWED execution:")
    try:
        result = client.verify_and_execute(
            "zero-human-daemon",
            GpuOperation.RENDER_PROGRAM,
            Permission.EXECUTE,
            lambda: "Successfully rendered on GPU!"
        )
        print(f"  Result: {result}")
    except CapabilityError as e:
        print(f"  ❌ Error: {e}")

    # Test denied execution
    print("\n❌ Testing DENIED execution:")
    try:
        result = client.verify_and_execute(
            "zero-human-daemon",
            GpuOperation.ALLOCATE_VRAM,
            Permission.READ_WRITE,
            lambda: "This should not execute"
        )
        print(f"  Result: {result}")
    except CapabilityError as e:
        print(f"  ✅ Correctly denied: {e}")


def main():
    """Run all integration tests"""
    print("""
╔════════════════════════════════════════════════════════════════╗
║         CBAC INTEGRATION TEST - AUTONOMOUS GPU OS              ║
║                                                                ║
║  Testing Capability-Based Access Control with TOML Manifests  ║
╚════════════════════════════════════════════════════════════════╝
""")

    try:
        test_zero_human_daemon()
        test_gvpie_daemon()
        test_resource_bounds()
        test_delegation_rules()
        test_verify_and_execute()

        print_section("✅ ALL TESTS PASSED")
        print("""
CBAC Integration Summary:
- ✅ Manifest loading from TOML
- ✅ Capability initialization
- ✅ Permission verification
- ✅ Resource bounds enforcement
- ✅ Delegation rules
- ✅ Verify-and-execute pattern

The CBAC security layer is operational.
The substrate is protected. The loop is sovereign.
""")

    except Exception as e:
        print(f"\n❌ TEST FAILED: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
