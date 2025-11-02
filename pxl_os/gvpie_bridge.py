import torch
import os
import mmap
import numpy as np

class GVPIEBridge:
    """
    Interface between the Python Pixel OS and the Rust/WGSL GVPIE environment.
    Uses a memory-mapped file for low-latency, zero-copy communication.
    """

    def __init__(self, file_path: str = "/tmp/gvpie_input.bin", size: int = 1024):
        self.file_path = file_path
        self.size = size
        self.mmap = self._init_mmap()

    def _init_mmap(self):
        """Initializes the memory-mapped file."""
        if os.path.exists(self.file_path):
            with open(self.file_path, "r+b") as f:
                return mmap.mmap(f.fileno(), self.size)
        else:
            with open(self.file_path, "wb") as f:
                f.write(b'\0' * self.size)
            with open(self.file_path, "r+b") as f:
                return mmap.mmap(f.fileno(), self.size)

    def send_command(self, command: str):
        """
        Sends a command to the GVPIE environment via the memory-mapped file.
        Protocol:
        - First byte is a flag: 1 = new command, 0 = no command.
        - The rest of the buffer is the null-terminated command string.
        """
        if self.mmap:
            # Ensure the first byte is 0 before writing.
            while self.mmap[0] != 0:
                pass

            # Write the command to the buffer.
            cmd_bytes = command.encode('utf-8')
            self.mmap[1:len(cmd_bytes) + 1] = cmd_bytes
            self.mmap[len(cmd_bytes) + 1] = 0 # Null terminator

            # Set the flag to indicate a new command is available.
            self.mmap[0] = 1
            self.mmap.flush()

    def read_output(self) -> str:
        """Reads the output from the GVPIE environment."""
        # This will be implemented in a later step.
        raise NotImplementedError

    def __del__(self):
        """Closes the memory-mapped file."""
        if self.mmap:
            self.mmap.close()
