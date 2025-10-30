# GVPIE Upgrade Checklist

This checklist keeps CPU↔GPU changes safe by enforcing the canonical I/O contract.

## Before editing shaders
- [ ] Review `src/io_contract.rs` for required layout or constant updates.
- [ ] Mirror any contract changes in `shaders/contract.wgsl`.
- [ ] Run `scripts/validate_contract.sh` to confirm Rust ↔ WGSL agreement.

## After editing shaders
- [ ] Run `cargo check` to ensure WGSL compiles through `naga`.
- [ ] Run `cargo test` (ensures buffer sizes and constants stay stable).
- [ ] Verify `RUST_LOG=info cargo run --release` in a graphical session.
- [ ] Manually test Ctrl+Space (3D toggle) and scroll behaviour.

## Common pitfalls
- Forgetting to update both Rust _and_ WGSL copies of the contract.
- Changing bind group layouts without adjusting `create_pipelines`.
- Introducing new buffers (remember the frozen bootstrap only has bindings 0‑3).
- Allowing event codes to drift from `io_contract` constants.

Keep this list close during future upgrades to avoid another bout of implicit contracts and distributed truth.
