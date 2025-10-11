# GPU Requirements

## Minimum Requirements (GVPIE v1.0)

GVPIE requires a WebGPU-capable GPU with these minimum specifications:

### Hardware
- **GPU Age:** 2012 or newer
- **GPU Type:** Discrete, Integrated, or Virtual GPU
- **API Support:** Vulkan 1.1, Metal 2, or DirectX 12

### Capabilities
- **Storage Buffer:** 128 MB minimum
- **Buffer Size:** 128 MB minimum
- **Compute Workgroups:** 256 threads minimum (X dimension)
- **Bind Groups:** 4 minimum

## Recommended Specifications

For optimal performance and full feature support:

- **Storage Buffer:** 1 GB or more
- **Buffer Size:** 1 GB or more
- **Compute Workgroups:** 1024 threads or more
- **GPU Type:** Discrete GPU
- **VRAM:** 2 GB or more

## Tested GPUs

### ✅ Known Compatible

**Desktop:**
- NVIDIA GeForce GTX 900 series and newer
- AMD Radeon RX 400 series and newer
- Intel Arc A-series

**Laptop:**
- NVIDIA GeForce GTX 1050 and newer
- AMD Radeon RX 5000M and newer
- Intel Iris Xe Graphics and newer

**Integrated:**
- Intel UHD Graphics 600 and newer
- AMD Radeon Vega 8 and newer
- Apple M1/M2/M3 GPUs

### ⚠️ Limited Support

**Software Renderers:**
- llvmpipe (CPU-based)
- SwiftShader

These work but are **extremely slow**. Only suitable for testing.

### ❌ Not Supported

- GPUs without WebGPU support
- GPUs with less than 128 MB storage buffer
- Pre-2012 graphics hardware
- GPUs without compute shader support

## Checking Your GPU

Run GVPIE with logging to see your GPU capabilities:

```bash
RUST_LOG=info cargo run --release
```

Look for the GPU validation section in the output:

```
✓ GPU meets all recommended requirements
```

or

```
⚠ Storage buffer below recommended: 256 MB (recommended: 1 GB)
```

## Platform-Specific Notes

### Linux
- Vulkan support recommended (install `vulkan-utils`)
- Mesa drivers should be 20.0 or newer
- Wayland and X11 both supported

### Windows
- DirectX 12 or Vulkan supported
- Update GPU drivers to latest version
- Windows 10/11 required

### macOS
- Metal 2 supported on macOS 10.15+
- All Apple Silicon Macs fully supported
- Intel Macs from 2012+ generally supported

## Troubleshooting

### "GPU does not meet minimum requirements"

Your GPU is too old or limited. Options:
1. Upgrade GPU hardware
2. Update GPU drivers to latest version
3. Check if GPU supports Vulkan/Metal/DX12

### "Software renderer detected"

You're using CPU-based rendering (llvmpipe). This is **very slow**.
- Check GPU drivers are installed
- Verify GPU is properly detected by OS
- Try `GVPIE_GPU=0` to select different adapter

### Poor Performance

If GVPIE runs but is slow:
1. Check GPU meets recommended specs (not just minimum)
2. Close other GPU-intensive applications
3. Update GPU drivers
4. Verify you're using discrete GPU (not integrated)

## Future Compatibility

The bootstrap is frozen at v1.0.0, so these requirements are **permanent**.

Future WGSL shader improvements may benefit from newer GPUs, but the
minimum requirements will never increase beyond what's listed here.

This ensures GVPIE remains compatible with a wide range of hardware
while the GPU-side code continues to evolve.
