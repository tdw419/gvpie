#!/usr/bin/env python3
"""
Quick test script for the GVPIE Bridge

Tests the connection to ai_runtime API and executes a simple pixel program.
"""

import asyncio
from pixel_os import GVPIEBridge, ExecutionBackend

async def test_bridge():
    """Test the GVPIE bridge"""

    print("=" * 60)
    print("🧪 GVPIE Bridge Test")
    print("=" * 60)
    print()

    async with GVPIEBridge() as bridge:
        # 1. Health check
        print("1️⃣  Checking API health...")
        healthy = await bridge.health_check()

        if not healthy:
            print("❌ API is not responding!")
            print()
            print("Please start ai_runtime:")
            print("  cd ../ai_runtime_rust")
            print("  cargo run --release")
            return

        print("✅ API is healthy")
        print()

        # 2. Get system status
        print("2️⃣  Getting system status...")
        status = await bridge.get_system_status()
        print(f"   Version: {status.get('version', 'unknown')}")
        print(f"   GPU Available: {status.get('gpu_available', False)}")
        print()

        # 3. Get available backends
        print("3️⃣  Checking available backends...")
        backends = await bridge.get_available_backends()
        print(f"   Available: {', '.join(backends)}")
        print()

        # 4. Execute a simple program
        print("4️⃣  Executing test pixel program...")
        program = """TXT 10 10 HELLO GPU
HALT"""

        print(f"   Program:\n{program}")
        print()

        result = await bridge.execute_program(
            source=program,
            backend=ExecutionBackend.GPU
        )

        if result.success:
            print(f"   ✅ Success!")
            print(f"   Cycles: {result.cycles_executed}")
            print(f"   Backend: {result.backend}")
            print(f"   Duration: {result.duration_ms}ms")
        else:
            print(f"   ❌ Failed: {result.error}")

        print()
        print("=" * 60)
        print("✅ Bridge test complete!")
        print("=" * 60)

if __name__ == "__main__":
    asyncio.run(test_bridge())
