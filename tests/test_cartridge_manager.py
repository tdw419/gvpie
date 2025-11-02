import unittest
from pathlib import Path
import numpy as np
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
from PIL import Image
from pxl_os.cartridge_manager import CartridgeManager, CartridgeType, CartridgeMetadata
from pxl_os.gvpie_bridge import GVPIEBridge

class TestCartridgeManager(unittest.TestCase):

    def setUp(self):
        self.bridge = GVPIEBridge()
        self.cm = CartridgeManager(self.bridge)
        self.test_dir = Path("tests/temp_cartridges")
        self.cm.cartridge_dir = self.test_dir
        self.test_dir.mkdir(exist_ok=True)

    def tearDown(self):
        for f in self.test_dir.glob("*.png"):
            f.unlink()
        self.test_dir.rmdir()

    def test_save_and_load_cartridge(self):
        canvas = np.zeros((64, 128, 3), dtype=np.uint8)
        metadata = CartridgeMetadata(
            type=CartridgeType.PIXEL,
            version="1.0",
            width=128,
            height=64,
            pages=1,
            encoding="rgb",
            checksum="test",
            created_at="now",
            tags=["test", "pixel"]
        )
        path = self.test_dir / "test_cartridge.png"

        self.cm.save_cartridge(canvas, metadata, path)
        self.assertTrue(path.exists())

        loaded_canvas, loaded_metadata = self.cm.load_cartridge(path)

        self.assertEqual(loaded_metadata.type, CartridgeType.PIXEL)
        self.assertEqual(loaded_metadata.tags, ["test", "pixel"])
        np.testing.assert_array_equal(canvas, loaded_canvas)

    def test_list_cartridges(self):
        # Create some dummy cartridges
        for i in range(3):
            path = self.test_dir / f"test_{i}.png"
            Image.fromarray(np.zeros((64, 128, 3), dtype=np.uint8)).save(path)

        cartridges = self.cm.list_cartridges()
        self.assertEqual(len(cartridges), 3)

if __name__ == "__main__":
    unittest.main()
