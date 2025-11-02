# GPU Computer Architecture

This document defines the canonical, unambiguous layer stack for the pixel-native AI OS. The architecture is defined from the bottom-up, starting with the host hardware and OS, and progressing to the high-level cognitive and multi-agent layers.

## System & Control Plane

*   **L0 — Host OS**: Linux (process, FS, input).
*   **L0.5 — GVPIE Bootstrap (Frozen)**: Rust + wgpu; creates the sovereign GPU context, owns machine/human textures, forwards input events.
*   **L1 — Daemon Runtime**: Python services (daemon, API), orchestration, policy.
*   **L2 — Persistence**: SQLite + content-addressed cartridges & logs.
*   **L3 — Color Semantics**: Shared color roles, themes, palette math.
*   **L3.5 — GPU Bridge / IPC**: Shared mem / ring buffers / file dropboxes for texture or command exchange.

## GPU Execution Plane (Peer Backends)

*   **L4 — Drivers/Runtime**: NVIDIA driver, CUDA runtime, Vulkan loader, wgpu.
*   **WGSL Path**
    *   **L5a — WGSL Shaders**: compute/render (GVPIE editor, glyph expander, scans).
    *   **L6a — SPIR-V**: compiled WGSL.
    *   **L7a — GPU ISA (via Vulkan)**: device code.
*   **CUDA Path**
    *   **L5b — CUDA Kernels**: e.g., `blit_string`, rectangle fills, scanners.
    *   **L6b — PTX/SASS**: compiled CUDA.
    *   **L7b — GPU ISA (via CUDA driver)**: device code.

## Pixel-Native OS & Above

*   **L8 — Pixel OS (PXL-ε)**: Canvas Manager, Fuzzy Font Engine, Fuzzy VM, Visual Opcodes, Hybrid Dispatcher (WGSL vs CUDA).
*   **L9 — Visual Tools**: Visual Editor, Debugger, confidence heatmaps, SSIM checks.
*   **L10 — Visual Language**: NL→pixel programs compiler + templates.
*   **L11 — Cognitive Core**: program synthesis/optimization/explanations.
*   **L12 — Multi-Agent / Sovereign Ecosystem**: coordination, provenance, shared visual KB.
