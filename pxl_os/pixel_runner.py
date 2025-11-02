from .pxl_epsilon_vm import PxlEpsilonVM
from .gvpie_bridge import GVPIEBridge

class PixelRunner:
    """
    The PixelRunner is responsible for dispatching operations to the optimal backend.
    """

    def __init__(self, vm: PxlEpsilonVM, bridge: GVPIEBridge):
        self.vm = vm
        self.bridge = bridge

    def execute(self, program: str, backend: str = "auto"):
        """
        Executes a pixel program on the specified backend.
        """
        if backend == "auto" or backend == "wgsl":
            # Default to the GVPIE bridge for now
            self.vm.execute(program)
        elif backend == "cuda":
            # CUDA implementation will be added later
            raise NotImplementedError
        else:
            raise ValueError(f"Unknown backend: {backend}")
