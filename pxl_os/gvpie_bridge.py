import torch
import os
from PIL import Image
import numpy as np

class GVPIEBridge:
    """
    Interface between the Python Pixel OS and the Rust/WGSL GVPIE environment.
    Uses a file-based IPC mechanism for communication.
    """

    def __init__(self, shared_dir: str = "/tmp/gvpie_ipc"):
        self.shared_dir = shared_dir
        self.machine_texture_path = os.path.join(shared_dir, "machine_texture.bin")
        self.human_texture_path = os.path.join(shared_dir, "human_texture.png")

        # Ensure the shared directory exists
        os.makedirs(self.shared_dir, exist_ok=True)

    def write_to_machine_texture(self, machine_code: torch.Tensor):
        """
        Sends text/commands to GVPIE's machine texture via a binary file.
        The machine_code tensor is expected to be (N, 3) with R=ASCII.
        """
        # Extract the red channel (ASCII codes) and write to a binary file.
        ascii_bytes = machine_code[:, 0].cpu().numpy().astype(np.uint8)
        with open(self.machine_texture_path, "wb") as f:
            f.write(ascii_bytes.tobytes())

    def read_from_human_texture(self) -> torch.Tensor:
        """
        Reads the expanded glyphs from GVPIE's human texture file.
        The GVPIE environment is expected to save its output as a PNG.
        Returns a torch.Tensor or None if the file doesn't exist.
        """
        if not os.path.exists(self.human_texture_path):
            return None

        try:
            with Image.open(self.human_texture_path) as img:
                img_array = np.array(img.convert("RGB"))
                return torch.from_numpy(img_array)
        except Exception as e:
            print(f"Error reading human texture: {e}")
            return None
