# pixel_os/cartridge_manager.py
"""
CartridgeManager: Bridge between your existing cartridge system and GVPIE
"""

import numpy as np
from PIL import Image, PngImagePlugin
from pathlib import Path
from typing import Optional, Dict, List, Tuple
from dataclasses import dataclass
from enum import Enum

from pxl_os.gvpie_bridge import GVPIEBridge
from pxl_os.fuzzy_font_engine import FuzzyFontEngine

# Cartridge types from your architecture
class CartridgeType(Enum):
    PIXEL = "pixel"           # Visual programs
    DATASET = "dataset"       # Multi-page data
    CODE = "code"            # Source code
    PROMPT = "prompt"        # Natural language
    HYBRID = "hybrid"        # Mixed content

@dataclass
class CartridgeMetadata:
    """Metadata from your cartridge spec"""
    type: CartridgeType
    version: str
    width: int
    height: int
    pages: int
    encoding: str
    checksum: str
    created_at: str
    tags: List[str]

class CartridgeManager:
    """
    Manages cartridge lifecycle:
    1. Load cartridge from PNG
    2. Parse metadata
    3. Convert to GVPIE-compatible format
    4. Execute on GPU
    5. Save results back as cartridge
    """

    def __init__(self, gvpie_bridge: GVPIEBridge):
        self.bridge = gvpie_bridge
        self.font_engine = FuzzyFontEngine(self.bridge)
        self.cartridge_dir = Path("cartridges")
        self.cartridge_dir.mkdir(exist_ok=True)

    # ===== LOADING =====

    def load_cartridge(self, path: Path) -> Tuple[np.ndarray, CartridgeMetadata]:
        """
        Load cartridge from PNG file.
        Extracts metadata from PNG tEXt chunks.
        """
        img = Image.open(path)
        canvas = np.array(img.convert('RGB'))

        # Extract metadata from PNG metadata chunks
        metadata = self._parse_metadata(img.info)

        return canvas, metadata

    def _parse_metadata(self, png_info: Dict) -> CartridgeMetadata:
        """Parse PNG tEXt metadata chunks"""
        return CartridgeMetadata(
            type=CartridgeType(png_info.get('cartridge_type', 'pixel')),
            version=png_info.get('version', '1.0'),
            width=int(png_info.get('width', 128)),
            height=int(png_info.get('height', 64)),
            pages=int(png_info.get('pages', 1)),
            encoding=png_info.get('encoding', 'rgb'),
            checksum=png_info.get('checksum', ''),
            created_at=png_info.get('created_at', ''),
            tags=png_info.get('tags', '').split(',')
        )

    # ===== CONVERSION =====

    def cartridge_to_program(self, canvas: np.ndarray, metadata: CartridgeMetadata) -> str:
        """
        Convert cartridge to executable pixel program.
        Uses fuzzy recognition to parse opcodes and arguments.
        """
        if metadata.type == CartridgeType.PIXEL:
            return self._parse_pixel_cartridge(canvas)
        elif metadata.type == CartridgeType.CODE:
            return self._parse_code_cartridge(canvas)
        elif metadata.type == CartridgeType.PROMPT:
            return self._parse_prompt_cartridge(canvas)
        else:
            raise ValueError(f"Unsupported cartridge type: {metadata.type}")

    def _parse_pixel_cartridge(self, canvas: np.ndarray) -> str:
        """
        Parse pixel cartridge using your opcode specification:
        - TXT opcode: 5×7 letters in 8×8 cell
        - Arguments: decimal digits
        - Scanline parsing (left-to-right, top-to-bottom)
        """
        program_lines = []
        height, width, _ = canvas.shape

        # Scan in 8×8 cells (your spec)
        for y in range(0, height, 8):
            line = ""
            for x in range(0, width, 8):
                cell = canvas[y:y+8, x:x+8, :]

                # Recognize opcode or text
                text, confidence = self.font_engine.recognize_cell(cell)

                if text and confidence > 0.6:  # Threshold from your spec
                    line += text
            if line:
                program_lines.append(line)

        return '\n'.join(program_lines)


    def _parse_code_cartridge(self, canvas: np.ndarray) -> str:
        """Parse code cartridge (plain text recognition)"""
        text, _ = self.font_engine.recognize_text(canvas, 0, 0, max_chars=1000)
        return text

    def _parse_prompt_cartridge(self, canvas: np.ndarray) -> str:
        """Parse prompt cartridge (natural language)"""
        return self._parse_code_cartridge(canvas)  # Same as code for now

    # ===== EXECUTION =====

    def execute_cartridge(self, path: Path) -> np.ndarray:
        """
        Complete cartridge execution pipeline:
        1. Load PNG
        2. Parse to program
        3. Execute on GVPIE
        4. Return result canvas
        """
        # Load
        canvas, metadata = self.load_cartridge(path)

        # Convert to program
        program = self.cartridge_to_program(canvas, metadata)

        # Execute on GPU
        if metadata.type == CartridgeType.PIXEL:
            # Send to GVPIE via bridge
            result = self._execute_on_gvpie(program)
        else:
            # Execute as Python/other
            result = self._execute_as_code(program)

        return result

    def _execute_on_gvpie(self, program: str) -> np.ndarray:
        """Execute pixel program on GVPIE"""
        # Write program to machine texture
        for i, line in enumerate(program.split('\n')):
            self.bridge.send_command(f"TXT 0 {i} {line}")

        # Read result from human texture
        # This is a placeholder, as the output is not yet implemented in the bridge
        return np.zeros((64, 128, 3), dtype=np.uint8)


    def _execute_as_code(self, code: str) -> np.ndarray:
        """Execute code cartridge (Python/SQL)"""
        # Execute in daemon context
        # Return visualization of result
        pass

    # ===== SAVING =====

    def save_cartridge(
        self,
        canvas: np.ndarray,
        metadata: CartridgeMetadata,
        path: Path
    ):
        """
        Save canvas as cartridge PNG with metadata.
        Uses PNG tEXt chunks for metadata storage.
        """
        img = Image.fromarray(canvas.astype(np.uint8))

        # Add metadata to PNG
        pnginfo = PngImagePlugin.PngInfo()
        pnginfo.add_text('cartridge_type', metadata.type.value)
        pnginfo.add_text('version', metadata.version)
        pnginfo.add_text('width', str(metadata.width))
        pnginfo.add_text('height', str(metadata.height))
        pnginfo.add_text('pages', str(metadata.pages))
        pnginfo.add_text('encoding', metadata.encoding)
        pnginfo.add_text('checksum', metadata.checksum)
        pnginfo.add_text('created_at', metadata.created_at)
        pnginfo.add_text('tags', ','.join(metadata.tags))

        img.save(path, pnginfo=pnginfo)
        print(f"✓ Saved cartridge: {path}")

    # ===== DISCOVERY =====

    def list_cartridges(self, filter_type: Optional[CartridgeType] = None) -> List[Path]:
        """List all cartridges in cartridge directory"""
        cartridges = []
        for path in self.cartridge_dir.glob("*.png"):
            if filter_type:
                try:
                    _, metadata = self.load_cartridge(path)
                    if metadata.type == filter_type:
                        cartridges.append(path)
                except Exception:
                    pass
            else:
                cartridges.append(path)
        return cartridges

    def search_cartridges(self, query: str) -> List[Path]:
        """Search cartridges by tags or content"""
        results = []
        for path in self.cartridge_dir.glob("*.png"):
            try:
                _, metadata = self.load_cartridge(path)
                if query.lower() in ' '.join(metadata.tags).lower():
                    results.append(path)
            except Exception:
                pass
        return results
