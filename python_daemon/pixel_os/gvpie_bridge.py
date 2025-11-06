"""
GVPIE Bridge - Python interface to the GPU AI Runtime

Provides a high-level API for executing pixel programs and managing cartridges.
"""

import aiohttp
import json
from typing import Dict, Any, List, Optional
from dataclasses import dataclass
from enum import Enum

class ExecutionBackend(Enum):
    """Execution backend for pixel programs"""
    GPU = "GPU"
    CPU = "CPU"
    AUTO = "Auto"

@dataclass
class PixelProgramResult:
    """Result of pixel program execution"""
    success: bool
    cycles_executed: int
    backend: str
    duration_ms: int
    canvas_data: Optional[List[int]] = None
    error: Optional[str] = None

class GVPIEBridge:
    """Bridge to ai_runtime API for GPU operations"""

    def __init__(self, api_url: str = "http://localhost:8081"):
        self.api_url = api_url
        self.session = None

    async def __aenter__(self):
        """Async context manager entry"""
        self.session = aiohttp.ClientSession()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager exit"""
        if self.session:
            await self.session.close()

    async def health_check(self) -> bool:
        """Check if the API is healthy"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        try:
            async with self.session.get(
                f"{self.api_url}/health",
                timeout=aiohttp.ClientTimeout(total=2)
            ) as resp:
                return resp.status == 200
        except Exception:
            return False

    async def get_system_status(self) -> Dict[str, Any]:
        """Get system status from API"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.get(f"{self.api_url}/status") as resp:
            return await resp.json()

    async def assemble_program(self, source: str) -> Dict[str, Any]:
        """Assemble pixel program source code into instructions"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.post(
            f"{self.api_url}/api/pixel/assemble",
            json={"source": source}
        ) as resp:
            return await resp.json()

    async def execute_program(
        self,
        source: str,
        backend: ExecutionBackend = ExecutionBackend.GPU,
        max_cycles: int = 1024,
        canvas_width: int = 128,
        canvas_height: int = 64
    ) -> PixelProgramResult:
        """Execute a pixel program"""

        # Assemble
        assemble_result = await self.assemble_program(source)

        if not assemble_result.get("success"):
            return PixelProgramResult(
                success=False,
                cycles_executed=0,
                backend="none",
                duration_ms=0,
                error=assemble_result.get("error", "Assembly failed")
            )

        program = assemble_result["program"]

        # Execute
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.post(
            f"{self.api_url}/api/pixel/run",
            json={
                "program": program,
                "backend": backend.value,
                "max_cycles": max_cycles,
                "canvas_width": canvas_width,
                "canvas_height": canvas_height
            }
        ) as resp:
            result = await resp.json()

            return PixelProgramResult(
                success=result.get("success", False),
                cycles_executed=result.get("cycles_executed", 0),
                backend=result.get("backend", "unknown"),
                duration_ms=result.get("duration_ms", 0),
                canvas_data=result.get("canvas_data"),
                error=result.get("error")
            )

    async def list_cartridges(self) -> List[Dict[str, Any]]:
        """List all available cartridges"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.get(f"{self.api_url}/api/cartridges") as resp:
            return await resp.json()

    async def create_cartridge(
        self,
        cartridge_id: str,
        name: str,
        description: str,
        code: str
    ) -> Dict[str, Any]:
        """Create a new cartridge"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.post(
            f"{self.api_url}/api/cartridges",
            json={
                "id": cartridge_id,
                "name": name,
                "description": description,
                "code": code
            }
        ) as resp:
            return await resp.json()

    async def get_cartridge(self, cartridge_id: str) -> Optional[Dict[str, Any]]:
        """Get a specific cartridge"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.get(
            f"{self.api_url}/api/cartridges/{cartridge_id}"
        ) as resp:
            if resp.status == 200:
                return await resp.json()
            return None

    async def execute_cartridge(self, cartridge_id: str) -> Dict[str, Any]:
        """Execute a cartridge by ID"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.post(
            f"{self.api_url}/api/execute",
            json={"code": cartridge_id}
        ) as resp:
            return await resp.json()

    async def get_available_backends(self) -> List[str]:
        """Get list of available execution backends"""
        if not self.session:
            self.session = aiohttp.ClientSession()

        async with self.session.get(f"{self.api_url}/api/pixel/backends") as resp:
            result = await resp.json()
            return result.get("backends", [])


# Convenience functions for synchronous use
async def quick_execute(source: str) -> PixelProgramResult:
    """Quick execution helper"""
    async with GVPIEBridge() as bridge:
        return await bridge.execute_program(source)


async def quick_health_check() -> bool:
    """Quick health check helper"""
    async with GVPIEBridge() as bridge:
        return await bridge.health_check()
