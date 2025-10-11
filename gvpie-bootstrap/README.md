# GVPIE Bootstrap v1.0.0 (Frozen)

The GVPIE bootstrap is the immutable CPU-side foundation of the GPU Sovereignty project. At exactly **1352 lines of Rust**, the host is now frozen: it launches the window, initialises WebGPU, validates the GPU, and forwards events into shared buffers. All editor logic, rendering, and future features live inside WGSL shaders that run entirely on the GPU.

## Quick Start

```bash
# ensure Rust 1.75+ is installed
cargo run --release
```

You should see:

```
✓ WebGPU instance created
✓ GPU meets all recommended requirements
✓ Buffers created (state/text/key/font)
✓ I/O buffers created (events: 4 KB, requests: 1 KB, file: 1024 KB)
```

Type into the window; every keystroke is processed by the GPU compute shader in `shaders/editor_compute.wgsl`.

## Hardware Requirements

| Capability                     | Minimum | Recommended |
|--------------------------------|---------|-------------|
| Storage buffer binding size    | 128 MB  | 1 GB        |
| Max buffer size                | 128 MB  | 1 GB        |
| Compute workgroup size (X)     | 256     | 1024        |
| Max bind groups                | 4       | 8           |

GPUs that do not meet the minimum requirements abort during startup with actionable guidance. See `docs/GPU_REQUIREMENTS.md` for tested hardware lists and troubleshooting tips.

## The Frozen Contract

The following files are **locked** and may only receive emergency bug fixes:

- `src/main.rs`
- `src/io_contract.rs`
- `src/gpu_requirements.rs`
- `Cargo.toml`

Every change must be represented by a new manifest in `docs/freeze.manifest` and tagged release. The current tag is `v1.0.0-frozen`.

### Host Responsibilities

1. Create the window + surface (winit + wgpu).
2. Select a GPU adapter and validate capabilities.
3. Allocate persistent GPU buffers:
   - Editor state & text
   - Circular key ring
   - Font atlas
   - I/O contract buffers (events, requests, file I/O)
4. Upload host events every frame and request redraws.

### GPU Responsibilities

Everything else: text editing, rendering, hotkeys, scrolling, future OS services— all authored in WGSL (`shaders/`).

## I/O Contract v1.0 Overview

Shared buffers live in GPU memory and define the perpetual CPU↔GPU interface:

- **Events Buffer** (`events: 4 KB`): Host → GPU input events (keyboard, resize, etc.).
- **Requests Buffer** (`requests: 1 KB`): GPU → Host service requests (file read/write, exit, etc.).
- **File I/O Buffer** (`file_io: 1 MiB`): Bidirectional payload for file contents.

The contract is versioned (`IO_CONTRACT_VERSION = 1`). Future extensions must maintain backward compatibility or ship side-by-side buffers. Full layout documented in `docs/IO_CONTRACT.md`.

## Project Layout

```
gvpie-bootstrap/
├── Cargo.toml
├── README.md
├── docs/
│   ├── FROZEN.md              # Official freeze declaration
│   ├── GPU_REQUIREMENTS.md    # Capability matrix and tested hardware
│   ├── IO_CONTRACT.md         # Buffer layout and semantics
│   └── freeze.manifest        # SHA-256 hashes of frozen files
├── shaders/
│   ├── editor_compute.wgsl    # Non-stop compute kernel (editor logic)
│   └── editor_render.wgsl     # Render pipeline (grid + cursor)
├── src/
│   ├── gpu_requirements.rs    # Capability validator
│   ├── io_contract.rs         # Frozen buffer definitions
│   └── main.rs                # Bootstrap entry point
├── tests/basic.gvp            # Example GVPIE program
└── FREEZE_RITUAL.md           # Narrative record of the freeze
```

## Development Model

All new behaviour is implemented by editing the WGSL shaders under `shaders/` and rerunning `cargo run --release`. The host binary will reload the compiled shaders on each execution.

Recommended workflow:

1. Modify `shaders/editor_compute.wgsl` to adjust logic (text manipulation, commands, etc.).
2. Modify `shaders/editor_render.wgsl` for visual updates.
3. Rebuild and run to observe changes.
4. If you must extend the I/O contract, bump `IO_CONTRACT_VERSION` and document changes in `docs/IO_CONTRACT.md` before touching Rust code.

## Environment Variables

| Variable       | Purpose                                             |
|----------------|------------------------------------------------------|
| `RUST_LOG`     | Configure logging (`RUST_LOG=info cargo run --release`). |
| `GVPIE_GPU`    | Override adapter selection (e.g. `GVPIE_GPU=1`).         |

## Freeze Manifest

`docs/freeze.manifest` contains the SHA-256 hashes of all frozen files. Any deviation indicates the bootstrap has been modified post-freeze and must result in a new version + tag.

```
3a7ff9ecbca953bcea800dc81464482148cda9a717ea402ab2799172114da8d1  src/main.rs
f51f2bb10730c68f8d7d20fbd2c9e83210fc99515b9ccf8e22b0f3fc01f3e585  src/io_contract.rs
da06bbfd07742ce7e1bd2a88bfbafe311155708c52dcbfbfb1cafe58dca17035  src/gpu_requirements.rs
```

## License

All code is released under the project’s main license (see repository root). Contributions must preserve GPU sovereignty by avoiding host changes.

---

**The CPU is frozen. The GPU is sovereign. Build the future in WGSL.**

