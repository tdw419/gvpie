# 🚀 GVPIE Quick Start Guide

## The Vision

You're building a **self-improving AI OS** that runs natively on GPU using visual programs. This guide gets you from zero to autonomous loop in **10 minutes**.

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  LAYER 13: Multi-Agent Ecosystem (Future)                   │
├─────────────────────────────────────────────────────────────┤
│  LAYER 12: Cognitive Core (Future)                          │
├─────────────────────────────────────────────────────────────┤
│  LAYER 11: Visual Language Compiler (Future)                │
├─────────────────────────────────────────────────────────────┤
│  LAYER 10: Visual Programming Interface (Future)            │
├─────────────────────────────────────────────────────────────┤
│  LAYER 9: Pixel OS (PXL-ε) - Python Daemon ⬅️ START HERE   │
├─────────────────────────────────────────────────────────────┤
│  LAYER 8: Fuzzy Font Engine (Python)                        │
├─────────────────────────────────────────────────────────────┤
│  LAYER 7: GPU Shader Interface (WGSL)                       │
├─────────────────────────────────────────────────────────────┤
│  LAYER 6: Machine Texture (RGB Codes)                       │
├─────────────────────────────────────────────────────────────┤
│  LAYER 5: Human Texture (Rendered Glyphs)                   │
├─────────────────────────────────────────────────────────────┤
│  LAYER 4: Glyph ROM (5×7 Bitmaps)                          │
├─────────────────────────────────────────────────────────────┤
│  LAYER 3: GPU Memory Manager (Rust)                         │
├─────────────────────────────────────────────────────────────┤
│  LAYER 2: wgpu Bindings (Rust)                             │
├─────────────────────────────────────────────────────────────┤
│  LAYER 1: Daemon Process (Rust API Server)                  │
├─────────────────────────────────────────────────────────────┤
│  LAYER 0.5: GVPIE Bootstrap (Rust + WGSL)                   │
├─────────────────────────────────────────────────────────────┤
│  LAYER 0: Host OS (Linux/Windows/macOS)                     │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 Prerequisites

- **Rust** 1.75+ (`cargo --version`)
- **Python** 3.8+ (`python3 --version`)
- **GPU** with Vulkan/Metal/DX12 support
- **Git** (you already have this)

---

## ⚡ 10-Minute Setup

### Step 1: Build the AI Runtime (2 minutes)

```bash
cd ai_runtime_rust
cargo build --release
```

This compiles:
- HTTP API server (port 8081)
- Pixel VM (interpreter for pixel programs)
- GPU bridge (connects to GVPIE bootstrap)
- Cartridge manager

### Step 2: Start the AI Runtime (1 minute)

**In Terminal 1:**
```bash
cd ai_runtime_rust
cargo run --release
```

You should see:
```
🌐 Server running on 0.0.0.0:8081
```

Keep this running!

### Step 3: Test the API (30 seconds)

**In Terminal 2:**
```bash
# Health check
curl http://localhost:8081/health

# Should return: ✅ AI Runtime Healthy
```

### Step 4: Set Up Python Daemon (1 minute)

```bash
cd python_daemon

# Install dependencies
pip3 install aiohttp

# Or use requirements.txt
pip3 install -r requirements.txt
```

### Step 5: Test the Bridge (30 seconds)

```bash
cd python_daemon
python3 test_bridge.py
```

You should see:
```
🧪 GVPIE Bridge Test
============================
1️⃣  Checking API health...
✅ API is healthy

2️⃣  Getting system status...
   Version: 0.1.0
   GPU Available: true

3️⃣  Checking available backends...
   Available: GPU, CPU

4️⃣  Executing test pixel program...
   Program:
   TXT 10 10 HELLO GPU
   HALT

   ✅ Success!
   Cycles: 2
   Backend: GPU
   Duration: 15ms
```

### Step 6: Launch the Autonomous Loop! (30 seconds)

```bash
cd python_daemon
./run_daemon.sh
```

You should see:
```
╔════════════════════════════════════════════════════════════════╗
║        🤖 ZERO HUMAN DAEMON - AUTONOMOUS AI OS 🤖              ║
╚════════════════════════════════════════════════════════════════╝

Starting autonomous development loop...

The daemon will:
  1. Execute improvement scripts
  2. Run pixel programs on GPU
  3. Save results as cartridges
  4. Learn from execution patterns
  5. Generate new improvements

Press Ctrl+C to stop.

================================================================
🤖 AUTONOMOUS CYCLE #1
================================================================
✅ API Health: ✅ AI Runtime Healthy

🚀 Executing 1 improvement script(s)

📝 Script: hello_gpu
   Language: pixel
   Purpose: Test GPU pipeline with simple text rendering
   ✅ Execution succeeded (2 cycles)
   ⏱️  Duration: 18ms

⏸️  Sleeping for 10s before next cycle...
```

**🎉 Congratulations! Your autonomous AI OS is now running!**

---

## 🔍 What Just Happened?

1. **AI Runtime API** (Rust) started listening on port 8081
2. **Python Daemon** connected to the API
3. **Improvement script** `hello_gpu` was loaded from database
4. **Pixel program** `TXT 10 10 HELLO GPU` was executed
5. **GPU** rendered the text using 5×7 glyphs
6. **Results** were saved to the infinite map
7. **Loop** continues every 10 seconds

---

## 📊 Monitoring Your System

### View the Infinite Map

```bash
sqlite3 python_daemon/db/daemon.db "
    SELECT
        iteration,
        task_description,
        substr(analysis, 1, 50) as analysis_preview,
        timestamp
    FROM infinite_map
    ORDER BY iteration DESC
    LIMIT 10
"
```

### Check Script Success Rates

```bash
sqlite3 python_daemon/db/daemon.db "
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

### Watch Live Logs

```bash
tail -f python_daemon/logs/daemon.log
```

---

## 🎯 Next Steps

### Phase 1: Validate the Pipeline (This Week)

- [x] ✅ AI Runtime API running
- [x] ✅ Python daemon connected
- [x] ✅ Pixel programs executing on GPU
- [x] ✅ Results saved to database
- [ ] 🔄 Run for 24 hours without crashes
- [ ] 🔄 Execute 100+ improvement scripts
- [ ] 🔄 Measure average latency (<50ms)

### Phase 2: Add Intelligence (Next Week)

- [ ] Connect to local LLM (Ollama)
- [ ] Generate new improvement scripts
- [ ] Implement self-healing
- [ ] Add visual debugger

### Phase 3: Go Autonomous (Week 3-4)

- [ ] Natural language → pixel compiler
- [ ] Multi-agent coordination
- [ ] Visual programming interface
- [ ] Self-modifying capabilities

---

## 🐛 Troubleshooting

### API Not Responding

```bash
# Check if it's running
curl http://localhost:8081/health

# Restart it
cd ai_runtime_rust
cargo run --release
```

### Database Locked

Only one daemon instance can run at a time:
```bash
# Find running instances
ps aux | grep zero_human_daemon

# Kill if needed
pkill -f zero_human_daemon
```

### GPU Not Available

The system will automatically fall back to CPU execution. Check:
```bash
# Vulkan support
vulkaninfo | head -20

# NVIDIA
nvidia-smi

# AMD
rocm-smi
```

---

## 📚 Key Concepts

### Pixel Programs (PXL-ε)

Visual programs that render directly to GPU canvas:

```
TXT 10 10 Hello World
RECT 5 5 100 20
HALT
```

### Improvement Scripts

Stored in database, executed automatically:
- `lang='pixel'` - Pixel programs (GPU visual)
- `lang='cartridge'` - Pre-compiled programs (PNG)
- `lang='python'` - Python code (sandboxed)

### Infinite Map

Complete history of all development activity:
- Every execution recorded
- Full lineage tracking
- Eternal memory for AI

### Autonomous Loop

```
While True:
    1. Check health
    2. Execute pending scripts
    3. Collect results
    4. Learn patterns
    5. Generate improvements
    6. Save to infinite map
    7. Sleep 10s
```

---

## 🎓 Learning Resources

- `python_daemon/README.md` - Detailed daemon documentation
- `gvpie-bootstrap/README.md` - GPU bootstrap details
- `docs/` - Architecture documentation
- `improvement_scripts/` - Example scripts

---

## 🤝 Development Workflow

### Add a New Improvement Script

```sql
-- Connect to database
sqlite3 python_daemon/db/daemon.db

-- Insert new script
INSERT INTO improvement_scripts (name, lang, purpose, code, enabled, created_at)
VALUES (
    'my_script',
    'pixel',
    'What it does',
    'TXT 20 20 My Text\nHALT',
    1,
    datetime('now')
);

-- The daemon will execute it automatically!
```

### Add a Development Goal

```sql
INSERT INTO development_goals (goal, priority, created_at)
VALUES ('Implement feature X', 10, datetime('now'));
```

### View Current Goals

```sql
SELECT * FROM development_goals
WHERE completed = 0
ORDER BY priority DESC;
```

---

## 🎉 You're Ready!

Your autonomous AI OS is now:
- ✅ Running on GPU
- ✅ Executing pixel programs
- ✅ Saving to infinite map
- ✅ Ready to evolve

**Watch it grow. Watch it learn. Watch it build itself.**

The age of autonomous AI development has begun. 🚀

---

## 🆘 Need Help?

- Check `QUICKSTART.md` (this file)
- Read `python_daemon/README.md`
- View logs in `python_daemon/logs/`
- Query database `python_daemon/db/daemon.db`

**The system documents itself. Just ask the infinite map.** 💫
