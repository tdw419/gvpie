import torch
import numpy as np
from .gvpie_bridge import GVPIEBridge
from .pixel_abi import GLYPH_ATLAS

class FuzzyFontEngine:
    """Bidirectional text to pixels with error tolerance."""

    def __init__(self, bridge: GVPIEBridge):
        self.bridge = bridge

    def render_text(self, x: int, y: int, text: str):
        """Renders text using the GVPIE bridge."""
        self.bridge.send_command(f"TXT {x} {y} {text}")

    def recognize_cell(self, cell: np.ndarray) -> (str, float):
        """Recongnizes a single 8x8 cell."""
        best_match = ' '
        highest_confidence = 0.0

        for char, glyph in GLYPH_ATLAS.items():
            confidence = self._calculate_confidence(cell, glyph)
            if confidence > highest_confidence:
                highest_confidence = confidence
                best_match = char

        return best_match, highest_confidence

    def recognize_text(self, canvas: np.ndarray, x: int, y: int, max_chars: int) -> (str, list[float]):
        """Recognizes text from a canvas with confidence scores."""
        recognized_text = ""
        confidences = []
        for i in range(max_chars):
            cell_x = x + i * 8
            if cell_x >= canvas.shape[1]:
                break

            cell = canvas[y:y+8, cell_x:cell_x+8, :]
            char, confidence = self.recognize_cell(cell)
            recognized_text += char
            confidences.append(confidence)

        return recognized_text, confidences

    def _calculate_confidence(self, cell: np.ndarray, glyph: list[list[int]]) -> float:
        """Calculates the confidence score for a given cell and glyph."""
        glyph_pixels = 0
        matching_pixels = 0

        for r, row in enumerate(glyph):
            for c, pixel in enumerate(row):
                if pixel == 1:
                    glyph_pixels += 1
                    # Check the corresponding pixel in the cell
                    # The glyph is centered in the 8x8 cell, so we need to offset
                    if cell[r, c + 1, 0] > 128: # Assuming white text on black background
                        matching_pixels +=1

        if glyph_pixels == 0:
            return 0.0

        return matching_pixels / glyph_pixels
