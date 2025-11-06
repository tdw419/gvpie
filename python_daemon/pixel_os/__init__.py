"""
Pixel OS - GPU-native visual programming system

This package provides the Python interface to the GPU AI OS via file-socket IPC.
"""

from .gvpie_bridge import GVPIEBridge, CANVAS_W, CANVAS_H
from .pixel_runner import PixelRunner

__all__ = [
    'GVPIEBridge',
    'PixelRunner',
    'CANVAS_W',
    'CANVAS_H',
]
