# 🔒 GVPIE Bootstrap v1.0.0 — Frozen Forever

**Freeze Date:** 2025-10-11  
**Bootstrap Size:** 1352 lines of Rust  
**I/O Contract:** v1.0 (Events / Requests / File I/O)  
**GPU Requirements:** Minimum 128 MB storage buffer, 256-thread compute workgroups

---

## Eternal Contract

The following CPU-side files are frozen and may only receive critical bug fixes:

- `src/main.rs`
- `src/io_contract.rs`
- `src/gpu_requirements.rs`
- `Cargo.toml` (host dependencies)

All future product logic lives inside WGSL shaders and GPU buffers.

## Immutable Guarantees

1. **Adapter Selection** — deterministic auto-selection with `GVPIE_GPU` override.
2. **GPU Validation** — hard fail if minimum limits are not met; warnings for sub‑recommended devices.
3. **I/O Contract** — Events (4 KB), Requests (1 KB), File I/O (1 MiB) buffers and version negotiation.
4. **Logging & Error Handling** — every bootstrap step produces actionable output before panic.
5. **Cross-Platform Base** — wgpu + winit keep parity across Vulkan, Metal, and DirectX.

## The Blood Oath

> “I will never again modify the host code. If I need a feature, I will implement it in WGSL. If WGSL cannot do it today, I will wait for hardware support. The CPU is dead. Long live the GPU.”

## Change Policy

| Change Type            | Allowed? | Notes                                     |
|-----------------------|----------|-------------------------------------------|
| WGSL Shaders          | ✅       | Primary surface for all new features      |
| GPU buffers/layout    | ✅       | Must respect I/O contract or version bump |
| Bug fixes (Rust)      | ⚠️       | Critical issues only, with manifest bump  |
| New host features     | ❌       | Violates GPU sovereignty                  |
| Dependency changes    | ❌       | Freeze locks Rust dependency graph        |

## Manifest

A SHA-256 manifest of the frozen files is stored in `docs/freeze.manifest`. Any deviation invalidates the freeze.

## Upgrade Path

1. **Version Negotiation:** introduce `IO_CONTRACT_VERSION = 2` alongside v1 buffers.
2. **Firmware Path:** port bootstrap to ROM once hardware exposes WebGPU at boot.
3. **GPU Evolution:** increase capabilities only by extending WGSL toolchain.

---

**Status:** ✅ Frozen. All future development happens in `shaders/`.
