# GVPIE Bootstrap: GPU-Native Development Environment

## 🚀 The Paradigm Shift

This is not just a text editor. This is a **proof-of-concept for GPU-sovereign computing** where the CPU is relegated to a thin, frozen bootloader and **all system logic runs on the GPU**.

### Traditional Architecture
```
CPU (Rust/C) → Controls Everything → GPU (Subordinate Accelerator)
```

### GVPIE Architecture
```
CPU (Frozen Bootstrap) → Initializes → GPU (Sovereign System Controller)
                ↓                              ↓
         OS Events Only              ALL Logic & State
```

## 🎯 What This Achieves

1. **Self-Hosting Editor**: The editor can edit its own WGSL source code
2. **GPU-Native Logic**: 100% of editor behavior defined in WGSL shaders
3. **Non-Stop Kernel**: Compute shader runs continuously, processing events
4. **Minimal CPU Dependency**: CPU only handles window creation and I/O proxy
5. **Foundation for GPU OS**: Basis for hypervisor, compiler, and full OS stack

## 📁 Project Structure

```
gvpie-bootstrap/
├── src/
│   └── main.rs              # FROZEN CPU bootstrap (~400 lines, never modified)
├── shaders/
│   ├── editor_compute.wgsl  # ALL editor logic (self-modifying)
│   └── editor_render.wgsl   # Text rendering pipeline
├── assets/                  # Future: font atlases, configs
└── Cargo.toml              # Rust dependencies
```

## 🔧 Building and Running

### Prerequisites

- Rust 1.70+ with cargo
- GPU with WebGPU support (Vulkan/Metal/DX12)
- For best results: AMD APU or Apple Silicon (Unified Memory Architecture)

### Build

```bash
cd gvpie-bootstrap
cargo build --release
```

### Run

```bash
cargo run --release
```

You should see a window with a GPU-native text editor. The welcome message confirms the GPU is now in control.

## 🎮 Editor Controls

- **Arrow Keys**: Navigate cursor
- **Home/End**: Jump to start/end of line
- **Backspace/Delete**: Remove characters
- **Enter**: New line
- **Tab**: Insert spaces
- **A-Z, 0-9, Space**: Type characters
- **Ctrl+S**: Save file (CPU-mediated I/O)

## 🧠 Architecture Deep Dive

### The Three Layers

#### Layer 1: Frozen CPU Bootstrap (`src/main.rs`)

**Role**: Minimal Trust Compute Base (MTCB)
**Responsibilities**:
- Initialize GPU context (wgpu)
- Create window surface (winit)
- Load WGSL shaders from files
- Marshal keyboard events → GPU ring buffer
- File I/O proxy (load/save only)

**Critical Property**: This code is **written once and never modified**. After Day 2, all development happens in WGSL.

#### Layer 2: GPU Compute Kernel (`shaders/editor_compute.wgsl`)

**Role**: System Controller & Editor Logic
**Responsibilities**:
- Process keyboard input events from ring buffer
- Maintain text buffer in GPU memory (UTF-32)
- Implement cursor movement, text insertion/deletion
- Track editor state (cursor position, line count, etc.)
- Run as Non-Stop Kernel (dispatched once, loops forever)

**Critical Property**: This is where **all editor behavior** is defined. Modify this shader to change how the editor works.

#### Layer 3: GPU Render Pipeline (`shaders/editor_render.wgsl`)

**Role**: Display & Visualization
**Responsibilities**:
- Read text buffer from GPU memory
- Render characters using bitmap font atlas
- Display cursor (blinking)
- Show line numbers
- Handle scrolling and viewport

**Critical Property**: Pure rendering - no logic. Reads state, displays output.

### Memory Architecture

```
GPU Memory (Unified on APU/Apple Silicon):
├── State Buffer (1KB)
│   ├── Cursor position (line, col)
│   ├── Scroll offset
│   ├── Text length & line count
│   └── Input ring buffer pointers
├── Text Buffer (40MB)
│   └── UTF-32 character array
├── Key Ring Buffer (1KB)
│   └── Circular buffer of keyboard events
└── Font Atlas (760 bytes)
    └── 8x8 bitmap for ASCII 32-126
```

### The Non-Stop Kernel Pattern

Traditional GPU compute:
```
CPU: dispatch_compute() → GPU: run_kernel() → complete
CPU: dispatch_compute() → GPU: run_kernel() → complete
...
```

GVPIE Non-Stop Kernel:
```
CPU: dispatch_compute() → GPU: while(running) { process_events(); }
                                      ↑______________|
                                   Never returns!
```

The compute shader runs **continuously**, checking for input events and updating state. This eliminates dispatch overhead and keeps the GPU in control.

### Synchronization Model

- **Atomic Operations**: All state updates use `atomic<u32>` for thread-safety
- **Storage Barriers**: `storageBarrier()` ensures memory visibility between compute and render
- **Ring Buffer**: Lock-free queue for CPU→GPU event communication

## 🔥 What Makes This Revolutionary

### 1. Self-Hosting Loop

```
Edit editor_compute.wgsl in the editor
         ↓
Save file (CPU writes to disk)
         ↓
Hot-reload shader (CPU recompiles)
         ↓
Editor now runs new behavior
         ↓
Repeat indefinitely
```

The editor can **modify its own source code** and immediately see the changes. This is the foundation for building a complete GPU-native development toolchain.

### 2. CPU Irrelevance

After the initial bootstrap, the CPU does **nothing except**:
- Forward keyboard events to GPU
- Save/load files on explicit command
- Refresh the display

The CPU is not executing any logic. It's a dumb I/O proxy.

### 3. Foundation for GPU Hypervisor

The same architecture extends to virtualization:

```
GPU Compute Kernel = Hypervisor
├── VM Scheduler (workgroup per VM)
├── Memory Manager (custom allocator)
├── Hypercall Handler (ring buffer)
└── I/O Virtualizer (DMA emulation)

Guest VMs = Parallel Compute Dispatches
```

## 🛠️ Next Steps (Weeks 2-4)

### Week 2: Self-Hosting Compiler

**Goal**: WGSL→SPIR-V compiler written in WGSL

```wgsl
// compiler.wgsl
@compute
fn compile_wgsl_to_spirv() {
    // Lexer: tokenize WGSL source
    // Parser: build AST
    // Codegen: emit SPIR-V bytecode
    // Store result in output buffer
}
```

This allows the editor to **compile and load new shaders** without CPU involvement.

### Week 3: GPU File System

**Goal**: Virtual file system in GPU storage buffers

```wgsl
struct File {
    name: array<u32, 64>,
    data: array<u32, 1000000>,
    size: u32,
}

@group(0) @binding(0) var<storage, read_write> files: array<File>;
```

Multiple shader files can coexist in GPU memory, edited and compiled independently.

### Week 4: Hypervisor Core

**Goal**: VM scheduler and memory isolation

```wgsl
struct VM {
    state: VMState,
    memory: array<u32, 10000000>,
    registers: array<u32, 32>,
}

@compute @workgroup_size(1)
fn vm_scheduler(@builtin(global_invocation_id) vm_id: vec3<u32>) {
    // Each workgroup is a VM
    // Process guest instructions
    // Handle hypercalls
    // Enforce memory isolation
}
```

## ⚠️ Current Limitations

### Hardware Requirements

- **Unified Memory**: Works best on APU/Apple Silicon where GPU and CPU share memory
- **Discrete GPUs**: Require data copying between VRAM and RAM (latency penalty)
- **Buffer Limits**: Some GPUs limit storage buffer sizes to 128MB (insufficient for large projects)

### Missing Features (Deliberate)

- **File Browser**: Coming in Week 2 (GPU-native)
- **Syntax Highlighting**: Coming in Week 2 (compute shader-based)
- **Multi-File Support**: Coming in Week 3 (GPU file system)
- **Copy/Paste**: Requires OS clipboard access (CPU mediation)
- **Mouse Input**: Not yet implemented (cursor positioning only)

### Known Issues

- **Font Rendering**: Currently uses minimal 8x8 bitmap font (placeholder)
- **Unicode Support**: UTF-32 storage but limited rendering (ASCII only for now)
- **Scroll Performance**: Recalculates line starts on every frame (optimization needed)
- **Preemption**: GPU kernel cannot be interrupted (requires hardware support)

## 🧪 Experimental Features

### Hot-Reloading (TODO)

Add file watching to automatically reload shaders when modified:

```rust
// In main.rs
use notify::{Watcher, RecursiveMode};

let mut watcher = notify::watcher(tx, Duration::from_secs(1))?;
watcher.watch("shaders/", RecursiveMode::Recursive)?;

// In event loop
if let Ok(DebouncedEvent::Write(path)) = rx.try_recv() {
    // Recompile and reload shader
    bootstrap.reload_shader(path);
}
```

### Debug Buffer

Add a debug output buffer to shader:

```wgsl
@group(0) @binding(4) var<storage, read_write> debug_log: array<u32>;

fn debug_print(msg: u32, value: u32) {
    let idx = atomicAdd(&debug_log[0], 1u);
    debug_log[idx * 2 + 1] = msg;
    debug_log[idx * 2 + 2] = value;
}
```

Read back on CPU for console logging.

## 📚 Further Reading

- **KGPU Paper**: "Augmenting Operating Systems With the GPU" (Non-Stop Kernel pattern)
- **WebGPU Spec**: https://www.w3.org/TR/webgpu/
- **WGSL Spec**: https://www.w3.org/TR/WGSL/
- **HSA Foundation**: Unified memory architectures

## 🤝 Contributing

This is an experimental research project. Key areas for contribution:

1. **Hardware Support**: Test on different GPUs, report compatibility
2. **Font Rendering**: Better bitmap fonts or SDF rendering
3. **Input Handling**: Mouse support, clipboard integration
4. **Performance**: Optimize line scanning, text insertion
5. **Features**: Undo/redo, search, syntax highlighting

## 📄 License

MIT License - Because this should be free as in freedom.

## 🙏 Acknowledgments

- **Timothy (GVPIE Vision)**: Original concept for GPU-first systems
- **KGPU Project**: Non-Stop Kernel pattern
- **WebGPU Working Group**: Modern GPU compute APIs

---

**Remember**: The CPU is frozen. The GPU is sovereign. All future development happens in WGSL.

Welcome to the future of systems programming. 🚀
