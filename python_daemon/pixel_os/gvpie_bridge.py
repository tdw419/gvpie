"""
GVPIE Bridge - File-socket IPC to GVPIE GPU renderer

Minimal bridge that talks to GVPIE via /tmp/gvpie/cmd.json
and reads from /tmp/gvpie/out.raw (RGBA canvas).
"""

import json
import time
from pathlib import Path

# Constants
GVPIE_DIR = Path("/tmp/gvpie")
CMD_PATH = GVPIE_DIR / "cmd.json"
OUT_PATH = GVPIE_DIR / "out.raw"

CANVAS_W = 128
CANVAS_H = 64

class GVPIEBridge:
    """File-based IPC bridge to GVPIE renderer."""

    def __init__(self, timeout: float = 2.0, poll: float = 0.02):
        """
        Args:
            timeout: Max wait time for GVPIE response (seconds)
            poll: Poll interval (seconds)
        """
        self.timeout = timeout
        self.poll = poll
        GVPIE_DIR.mkdir(parents=True, exist_ok=True)

    def _write_cmd(self, cmd_dict: dict):
        """Write command to /tmp/gvpie/cmd.json"""
        CMD_PATH.write_text(json.dumps(cmd_dict))

    def _wait_for_output(self) -> bytes:
        """Wait for /tmp/gvpie/out.raw to appear, then read it."""
        if OUT_PATH.exists():
            OUT_PATH.unlink()

        deadline = time.time() + self.timeout
        while time.time() < deadline:
            if OUT_PATH.exists():
                raw = OUT_PATH.read_bytes()
                OUT_PATH.unlink()
                return raw
            time.sleep(self.poll)

        raise TimeoutError(f"GVPIE did not respond within {self.timeout}s")

    def render_program(self, code: str) -> bytes:
        """
        Render a pixel program, return raw RGBA bytes (128×64×4).

        Args:
            code: Pixel program source (e.g., "TXT 10 10 Hello\\nHALT")

        Returns:
            Raw RGBA bytes (32768 bytes for 128×64 canvas)
        """
        self._write_cmd({
            "op": "render_program",
            "code": code,
            "width": CANVAS_W,
            "height": CANVAS_H,
            "format": "RGBA",
        })
        return self._wait_for_output()

    def write_text(self, x: int, y: int, text: str) -> bytes:
        """Convenience: render a single TXT opcode."""
        code = f"TXT {x} {y} {text}\nHALT"
        return self.render_program(code)

    def read_canvas(self) -> bytes:
        """Read current canvas state (if GVPIE supports this)."""
        self._write_cmd({"op": "read_canvas"})
        return self._wait_for_output()

    def health_check(self) -> bool:
        """Check if GVPIE is responding."""
        try:
            self._write_cmd({"op": "ping"})
            deadline = time.time() + self.timeout
            while time.time() < deadline:
                if OUT_PATH.exists():
                    OUT_PATH.unlink()
                    return True
                time.sleep(self.poll)
            return False
        except Exception:
            return False
