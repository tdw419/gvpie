"""
Pixel OS - GPU-native visual programming system

This package provides the Python interface to the GPU AI OS.
"""

from .gvpie_bridge import (
    GVPIEBridge,
    ExecutionBackend,
    PixelProgramResult,
    quick_execute,
    quick_health_check
)

__all__ = [
    'GVPIEBridge',
    'ExecutionBackend',
    'PixelProgramResult',
    'quick_execute',
    'quick_health_check'
]
