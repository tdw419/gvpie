# Zero Human Daemon - Autonomous AI OS

The Python orchestrator for the GPU-native AI operating system.

## Architecture

```
┌─────────────────────────────────────────┐
│   zero_human_daemon.py                  │
│   (Orchestration & Autonomous Loop)     │
└────────────┬────────────────────────────┘
             │
             │ HTTP API
             │
┌────────────▼────────────────────────────┐
│   ai_runtime (Rust)                     │
│   - Pixel VM                            │
│   - GPU Bridge                          │
│   - Cartridge Manager                   │
└────────────┬────────────────────────────┘
             │
             │ wgpu
             │
┌────────────▼────────────────────────────┐
│   GVPIE Bootstrap (Rust + WGSL)         │
│   - Machine Texture (RGB codes)         │
│   - Human Texture (rendered glyphs)     │
│   - 5×7 Glyph Expansion                 │
└─────────────────────────────────────────┘
             │
             │
┌────────────▼────────────────────────────┐
│   GPU (CUDA/Vulkan/Metal)               │
└─────────────────────────────────────────┘
```

## Quick Start

### 1. Start the AI Runtime API (in separate terminal)

```bash
cd ../ai_runtime_rust
cargo run --release
# Server will start on http://localhost:8081
```

### 2. Initialize the Database

```bash
cd python_daemon
python3 zero_human_daemon.py  # This will auto-create the database

# Ctrl+C to stop after database is initialized
```

### 3. Load Initial Improvement Scripts

```bash
sqlite3 db/daemon.db < improvement_scripts/001_hello_gpu.sql
```

### 4. Run the Autonomous Loop

```bash
python3 zero_human_daemon.py
```

The daemon will:
- ✅ Check API health
- ✅ Execute pending improvement scripts
- ✅ Run pixel programs on GPU
- ✅ Save results to infinite map
- ✅ Generate new improvements (coming soon)

## Database Schema

### Tables

- **improvement_scripts**: Scripts to execute (pixel programs, Python code, cartridges)
- **improvement_runs**: Execution history with results
- **development_goals**: What the AI should build next
- **infinite_map**: Complete history of all development activity
- **cartridge_registry**: Metadata about saved cartridges
- **system_metrics**: Performance and health metrics

## Improvement Scripts

Scripts are stored in the database with:
- `name`: Unique identifier
- `lang`: Language (pixel, cartridge, python)
- `code`: The actual code to execute
- `purpose`: Description of what it does
- `enabled`: Whether to run it (0/1)

### Example Pixel Program

```sql
INSERT INTO improvement_scripts (name, lang, purpose, code, enabled, created_at)
VALUES (
    'draw_rectangle',
    'pixel',
    'Draw a colored rectangle',
    'RECT 10 10 50 30
    HALT',
    1,
    datetime('now')
);
```

## Pixel Language (PXL-ε)

### Opcodes

- `TXT x y text` - Draw text at position
- `RECT x y w h` - Draw rectangle
- `HALT` - Stop execution

More opcodes coming: `JMP`, `CALL`, `SET`, `IF`, etc.

## The Autonomous Loop

Every 10 seconds (configurable):

1. **Check Health**: Verify AI Runtime API is responsive
2. **Execute Scripts**: Run pending improvement scripts
3. **Collect Results**: Record execution metrics
4. **Learn**: Analyze patterns and failures
5. **Generate**: Create new improvements (Phase 2)
6. **Repeat**: Continue forever

## Infinite Map

Every execution is recorded in the `infinite_map` table, creating a complete history of the system's development.

Query the map:

```sql
SELECT
    iteration,
    task_description,
    substr(result, 1, 50) as result_preview,
    timestamp
FROM infinite_map
ORDER BY iteration DESC
LIMIT 10;
```

## Next Steps

Phase 1 (Current):
- [x] Basic daemon loop
- [x] Pixel program execution
- [x] Database schema
- [ ] Cartridge execution
- [ ] Python sandbox execution

Phase 2 (Week 2):
- [ ] Connect to local LLM (Ollama/LM Studio)
- [ ] Generate improvements autonomously
- [ ] Self-healing error correction
- [ ] Performance optimization

Phase 3 (Week 3-4):
- [ ] Visual debugger
- [ ] Visual editor
- [ ] Multi-agent coordination
- [ ] Natural language → pixel compiler

## Configuration

Environment variables:
- `GVPIE_API_URL`: AI Runtime API URL (default: http://localhost:8081)
- `GVPIE_DB_PATH`: Database path (default: db/daemon.db)
- `GVPIE_CYCLE_INTERVAL`: Seconds between cycles (default: 10)

## Monitoring

View live metrics:

```bash
# Watch the logs
tail -f logs/daemon.log

# Query metrics
sqlite3 db/daemon.db "SELECT * FROM system_metrics ORDER BY timestamp DESC LIMIT 20"

# Check script success rates
sqlite3 db/daemon.db "
    SELECT
        name,
        run_count,
        success_count,
        CAST(success_count AS FLOAT) / run_count * 100 as success_rate
    FROM improvement_scripts
    WHERE run_count > 0
    ORDER BY success_rate DESC
"
```

## Troubleshooting

**API not responding:**
```bash
# Check if ai_runtime is running
curl http://localhost:8081/health

# Start it if needed
cd ../ai_runtime_rust && cargo run --release
```

**Database locked:**
```bash
# Only one daemon instance can run at a time
ps aux | grep zero_human_daemon
# Kill other instances if needed
```

**GPU not available:**
The system will fall back to CPU execution automatically.

## Contributing

This is an autonomous system - it contributes to itself!

But you can:
1. Add improvement scripts to the database
2. Define new development goals
3. Monitor the infinite map for interesting patterns
4. Bless successful improvements

---

**Welcome to the future of autonomous AI development.** 🚀
