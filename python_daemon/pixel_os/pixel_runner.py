"""
Pixel Runner - Wraps GVPIE bridge and saves cartridges

Converts raw RGBA bytes from GVPIE into PNG cartridges with metadata.
"""

import hashlib
import time
from pathlib import Path
from PIL import Image
from PIL import PngImagePlugin

from .gvpie_bridge import GVPIEBridge, CANVAS_W, CANVAS_H

class PixelRunner:
    """High-level interface for executing pixel programs and saving cartridges."""

    def __init__(self, timeout: float = 2.0):
        self.bridge = GVPIEBridge(timeout=timeout)

    def execute_text_program(self, code: str) -> Image.Image:
        """
        Execute a pixel program and return PIL Image.

        Args:
            code: Pixel program source

        Returns:
            PIL Image (RGBA, 128×64)
        """
        raw = self.bridge.render_program(code)
        img = Image.frombytes("RGBA", (CANVAS_W, CANVAS_H), raw)
        return img

    def save_cartridge(self, img: Image.Image, path: str | Path):
        """
        Save image as PNG cartridge with metadata.

        Adds tEXt chunks:
          - cartridge_type: "pixel"
          - checksum: SHA-256 of raw image data
          - created_at: Unix timestamp
        """
        path = Path(path)
        path.parent.mkdir(parents=True, exist_ok=True)

        # Prepare metadata
        pnginfo = PngImagePlugin.PngInfo()
        pnginfo.add_text("cartridge_type", "pixel")

        # Compute checksum
        data_bytes = img.tobytes()
        checksum = hashlib.sha256(data_bytes).hexdigest()
        pnginfo.add_text("checksum", checksum)

        # Timestamp
        pnginfo.add_text("created_at", str(int(time.time())))

        # Save
        img.save(path, format="PNG", pnginfo=pnginfo)

    def execute_and_save(self, code: str, path: str | Path) -> Image.Image:
        """
        Execute pixel program and save as cartridge.

        Args:
            code: Pixel program source
            path: Output PNG path

        Returns:
            PIL Image
        """
        img = self.execute_text_program(code)
        self.save_cartridge(img, path)
        return img
