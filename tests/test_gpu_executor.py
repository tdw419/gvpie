import unittest
import subprocess
import time
import os

class TestGpuExecutor(unittest.TestCase):

    def test_gpu_executor_hello_world(self):
        # Compile and run the gvpie-bootstrap executable
        process = subprocess.Popen(
            ["cargo", "run", "--package", "gvpie-bootstrap"],
            cwd="gvpie-bootstrap",
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )

        # Give the process time to start up
        time.sleep(5)

        # Send a command to the IPC file
        with open("/tmp/gvpie/control.bin", "w") as f:
            f.write("RED")

        # Give the process time to react
        time.sleep(1)

        # Check the log output
        stdout, stderr = process.communicate()
        log_output = stdout.decode("utf-8") + stderr.decode("utf-8")

        self.assertIn("File command: RED", log_output)

        # Clean up
        process.kill()

if __name__ == "__main__":
    unittest.main()
