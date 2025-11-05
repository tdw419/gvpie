# Auto-GVPIE: Autonomous Development Loop

**GVPIE builds itself using local LLM + LanceDB + continuous feedback**

## 🎯 Concept

Instead of you writing every line of code, Auto-GVPIE:

1. Picks tasks from a queue
2. Asks your local LLM (LM Studio) to implement them
3. Applies patches and tests
4. Stores results in LanceDB for memory
5. Feeds errors back to LLM for fixes
6. Outputs pixel representations
7. Repeats forever

**Result**: GVPIE evolves autonomously on your machine.

## 🏗️ Architecture

```
┌──────────────┐
│  Task Queue  │  (auto_tasks.json)
└──────┬───────┘
       │
       ↓
┌──────────────┐
│   LM Studio  │  (localhost:1234)
│  Local LLM   │
└──────┬───────┘
       │ generates
       ↓
┌──────────────┐
│   Patches    │  (code or pixel specs)
│  + Specs     │
└──────┬───────┘
       │ apply
       ↓
┌──────────────┐
│  Test & Run  │  (cargo check, tests)
└──────┬───────┘
       │ results
       ↓
┌──────────────┐
│   LanceDB    │  (memory + RAG)
│   + Files    │
└──────┬───────┘
       │ context for next round
       └────────┐
                ↓
         ┌──────────────┐
         │  Pixel View  │  (visual output)
         └──────────────┘
```

## 📦 Components

### 1. `orchestrator.py` - Main Loop

The brain. Continuously:
- Loads tasks
- Calls LLM
- Applies patches
- Runs tests
- Stores in LanceDB
- Handles errors

### 2. `view_pixels.py` - Pixel Viewer

Visualizes what the LLM built:
- Reads pixel programs from LanceDB or JSON
- Renders ASCII art
- Saves PNG images
- Shows ops and metadata

### 3. `auto_tasks.json` - Task Queue

JSON file with pending work:

```json
[
  {
    "id": "task-001",
    "title": "Add CLICK_AT opcode to PixelRunner",
    "inputs": ["src/pixel_runner.rs"],
    "tests": ["cargo test pixel_runner::click_at"],
    "status": "pending"
  }
]
```

### 4. LanceDB - Memory

Stores:
- **tasks**: what to build
- **artifacts**: LLM output (specs, patches, pixel programs)
- **runs**: test results, errors, logs

Later LLM calls can RAG from this to improve.

## 🚀 Quick Start

### Prerequisites

1. **LM Studio running at http://127.0.0.1:1234**
   - Download from https://lmstudio.ai
   - Load a code model (e.g., CodeLlama, DeepSeek Coder)
   - Start local server on port 1234

2. **Python dependencies**:
   ```bash
   pip install requests lancedb pillow
   ```

### Running

```bash
# Run once (process pending tasks)
python auto_gvpie/orchestrator.py --once

# Run with max tasks
python auto_gvpie/orchestrator.py --max-tasks 5

# View latest pixel program
python auto_gvpie/view_pixels.py --latest

# View specific artifact
python auto_gvpie/view_pixels.py --file artifacts/art-abc123.json

# List all artifacts
python auto_gvpie/view_pixels.py --list
```

### Adding Tasks

Edit `auto_tasks.json`:

```json
{
  "id": "task-new",
  "title": "Your task description",
  "inputs": ["path/to/file.rs"],
  "tests": ["cargo test", "cargo check"],
  "status": "pending"
}
```

Types of tasks:

**Code tasks** (LLM generates patches):
- "Add error handling to X"
- "Implement feature Y"
- "Fix bug in Z"

**Pixel tasks** (LLM generates pixel specs):
- "Create boot screen visual"
- "Design menu interface"
- "Make progress indicator"

## 🎨 Pixel Programs

The LLM can generate visual elements as JSON:

```json
{
  "kind": "pixel_program",
  "version": 1,
  "name": "boot_screen",
  "canvas": {"width": 64, "height": 64},
  "palette": {
    "bg": [0, 0, 0],
    "fg": [0, 255, 0]
  },
  "ops": [
    {"op": "FILL", "color": "bg"},
    {"op": "RECT", "x": 10, "y": 10, "width": 44, "height": 44, "color": "fg"},
    {"op": "WRITE_TEXT", "x": 20, "y": 28, "text": "GVPIE", "color": "fg"}
  ]
}
```

These get:
1. Stored in LanceDB
2. Saved as JSON in `artifacts/`
3. Rendered to PNG by `view_pixels.py`
4. Eventually executed on GPU

## 🔄 The Feedback Loop

When a test fails:

1. Capture stderr
2. Send back to LLM: "Here's the error, fix it"
3. LLM generates new patch
4. Apply and test again
5. Store both attempts in LanceDB

This creates **self-improving behavior**.

## 💾 Memory & RAG

LanceDB stores everything:

```python
# Later, LLM can query context:
"What did I try for pixel_runner before?"
"Show me previous attempts at CLICK_AT"
"What errors occurred in the last 10 builds?"
```

This makes each iteration smarter than the last.

## 🎯 GVPIE-Specific Features

### GPU-Sovereign Tasks

```json
{
  "title": "Optimize GPU pixel analyzer for larger programs",
  "inputs": ["shaders/pixel_analyzer.wgsl"],
  "tests": ["cargo run --example gpu_sovereign_analyzer"]
}
```

### VRAM-Based Tasks

```json
{
  "title": "Create VRAM-backed code editor pixel program",
  "inputs": [],
  "tests": []
}
```

### Pixel System Tasks

```json
{
  "title": "Design file browser interface as pixel program",
  "inputs": [],
  "tests": []
}
```

## 🔧 Integration with Daemon

Your existing daemon can:

1. **Watch** `auto_tasks.json` for new tasks
2. **Trigger** `orchestrator.py` when tasks added
3. **Run** continuously every N minutes
4. **Monitor** LanceDB for completed work

```python
# In your daemon:
import subprocess

def auto_gvpie_cycle():
    subprocess.run(["python", "auto_gvpie/orchestrator.py", "--once"])
```

## 📊 What Gets Stored

Every cycle creates:

```
artifacts/
  art-abc123.json     # LLM-generated pixel program
  art-abc123.png      # Rendered visualization
  run-xyz789.log      # Test output

lancedb/
  tasks/              # All tasks (pending, done, failed)
  artifacts/          # All LLM outputs (searchable)
  runs/               # All test runs (for learning)
```

## 🎯 Next Steps

1. **Add more task templates** (common patterns)
2. **Implement fix-retry loop** (automatic error recovery)
3. **Add judge agent** (second LLM reviews first)
4. **Enable RAG queries** (LLM pulls from LanceDB)
5. **GPU task execution** (run pixel programs on GPU)
6. **Visual OS integration** (pixel programs become UI)

## 🚦 Status Tracking

Tasks flow through states:

```
pending → in_progress → done
                     ↘ failed (with error stored)
```

View status:

```bash
python auto_gvpie/orchestrator.py --status
```

## 🎨 Example Workflow

```bash
# 1. Start LM Studio (port 1234)

# 2. Add a task
echo '{
  "id": "task-cursor",
  "title": "Create animated cursor pixel program",
  "inputs": [],
  "tests": [],
  "status": "pending"
}' >> auto_tasks.json

# 3. Run auto-build
python auto_gvpie/orchestrator.py --once

# 4. View the result
python auto_gvpie/view_pixels.py --latest

# Output:
#   ✅ Pixel program: animated_cursor
#   ✅ ASCII preview
#   ✅ Saved to: art-abc123.png
```

## 🔄 Continuous Operation

For true autonomy, run in a loop:

```python
# run_forever.py
import time
from auto_gvpie.orchestrator import AutoGVPIE

auto = AutoGVPIE()
while True:
    auto.run_loop(max_tasks=1)
    time.sleep(60)  # Check every minute
```

Or integrate with your existing daemon's event loop.

## 🎯 The Vision

**GVPIE builds itself autonomously:**

- You add high-level tasks
- Local LLM implements them
- System tests and stores
- Errors auto-fix
- Pixel OS grows visually
- All in VRAM eventually

**Result**: Development continues 24/7 on your machine, building toward GPU sovereignty.

---

**This is Auto-GVPIE: GVPIE building GVPIE.** 🚀
