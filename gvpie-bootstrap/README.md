# GVPIE Bootstrap

This crate is a minimal CPU bootstrap that hands control to GPU-resident shaders.

## Prerequisites

- Rust 1.75+ (with `cargo`)
- A GPU/device that supports the [`wgpu`](https://github.com/gfx-rs/wgpu) 0.19 backend (Vulkan, Metal, DX12, or WebGPU on compatible platforms)

## Running

```bash
cargo run --release
```

The first launch compiles the Rust host and the WGSL shaders found in `shaders/editor_compute.wgsl` and `shaders/editor_render.wgsl`. When the window appears, all text-editing logic runs inside the compute shader; the Rust process only marshals events and presents frames.

## Project Layout

- `src/main.rs` – frozen host bootstrap that initialises wgpu, manages the window, and forwards keyboard events to the GPU.
- `shaders/editor_compute.wgsl` – non-stop compute kernel that maintains editor state, processes key events, and mutates the UTF-32 text buffer.
- `shaders/editor_render.wgsl` – fullscreen render pipeline that reads the text buffer and displays it with a bitmap font.

Modify the WGSL files to change behaviour; rebuilding the binary is only required if the host bootstrap itself changes.
