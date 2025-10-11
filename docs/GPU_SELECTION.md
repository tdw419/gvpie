# GPU Adapter Selection

GVPIE boots with a frozen policy for choosing the GPU adapter. The bootstrap never changes; all configuration happens through environment variables.

## Automatic Selection (Default)

```bash
RUST_LOG=info cargo run --release
```

The bootstrap prints every available adapter, then picks the best option in priority order:

1. Discrete GPU (maximum performance)
2. Integrated GPU (battery friendly)
3. First available adapter (fallback)

## Manual Override

Force a specific GPU by index:

```bash
# Use first adapter (typically discrete)
GVPIE_GPU=0 RUST_LOG=info cargo run --release

# Use second adapter (typically integrated)
GVPIE_GPU=1 RUST_LOG=info cargo run --release
```

Indices start at `0`. Run with logging to see the full list.

## Viewing Available GPUs

```bash
RUST_LOG=info cargo run --release
```

Example output:

```
Available GPU Adapters: 2
  [0] NVIDIA GeForce RTX 3080 - Discrete GPU (Vulkan)
  [1] Intel UHD Graphics 630 - Integrated GPU (Vulkan)

✓ Auto-selected: NVIDIA GeForce RTX 3080 (discrete GPU (best performance))
```

## Common Scenarios

- **Battery saving (laptops)**  
  `GVPIE_GPU=1 cargo run --release`

- **Performance comparisons**  
  ```
  GVPIE_GPU=0 cargo run --release
  GVPIE_GPU=1 cargo run --release
  ```

- **Debugging GPU issues**  
  `RUST_LOG=debug GVPIE_GPU=1 cargo run`

- **CI sweeps across adapters**  
  ```bash
  for i in 0 1 2 3; do
      echo "Testing GPU $i..."
      GVPIE_GPU=$i cargo run --release || echo "GPU $i: FAILED"
  done
  ```

## Troubleshooting

- **“No GPU adapters found”**  
  Check drivers, confirm Vulkan/Metal/DirectX support, try vendor diagnostics.

- **“Selected GPU cannot render to window surface”**  
  Pick another GPU (`GVPIE_GPU=0`), update drivers, verify monitor setup.

- **“GVPIE_GPU out of range”**  
  Inspect the logged adapter list; valid indices are `0..N-1`.
