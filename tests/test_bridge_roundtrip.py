import subprocess
import time
import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

from pxl_os.gvpie_bridge import GVPIEBridge

def test_bridge_roundtrip():
    """
    An end-to-end integration test for the GVPIE bridge.
    """
    # Build the gvpie-bootstrap executable
    build_process = subprocess.run(["cargo", "build"], cwd="gvpie-bootstrap")
    assert build_process.returncode == 0

    # Start the gvpie-bootstrap process in the background
    env = os.environ.copy()
    env["GVPIE_HEADLESS"] = "1"
    process = subprocess.Popen(
        ["./target/debug/gvpie-bootstrap"],
        cwd="gvpie-bootstrap",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env,
    )

    # Wait for the process to start up
    time.sleep(2)

    # Send a command through the bridge
    bridge = GVPIEBridge()
    bridge.send_command("Hello, World!")

    # Wait for the command to be processed
    time.sleep(2)

    # Stop the process
    process.terminate()

    # Check the output
    stdout, stderr = process.communicate()

    assert "File command: [72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]" in stdout

if __name__ == "__main__":
    test_bridge_roundtrip()
