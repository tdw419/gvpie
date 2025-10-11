# GVPIE Technical Architecture

## Executive Summary

This document describes the complete architecture of the GVPIE (Graphics Virtual Processor Infrastructure Environment) Bootstrap, a proof-of-concept for GPU-sovereign computing where traditional CPU-centric control flow is inverted.

## Architectural Philosophy

### The Sovereignty Principle

**Traditional Computing**: CPU is the master, GPU is the servant
**GVPIE Computing**: GPU is sovereign, CPU is the minimal bootloader

This isn't just an optimization—it's a fundamental rethinking of the compute hierarchy.

## System Components

### 1. The Minimal Trust Compute Base (MTCB)

**File**: `src/main.rs`
**Lines of Code**: ~400
**Language**: Rust (frozen after Day 2)

#### Responsibilities

1. **Initialization**: 
   - Create GPU context via wgpu
   - Allocate persistent GPU buffers
   - Load WGSL shaders from disk
   - Configure rendering pipeline

2. **Event Marshaling**:
   - Capture OS keyboard events (winit)
   - Serialize to GPU-readable format
   - Write to GPU ring buffer

3. **I/O Proxy**:
   - Read text buffer from GPU memory
   - Write to persistent storage on save command
   - Load files into GPU memory

4. **Display Management**:
   - Configure window surface
   - Handle resize events
   - Trigger render pipeline

#### Key Property: Immutability

After initial development (Day 1-2), this code **never changes**. All future development happens in WGSL shaders. This is enforced by:

- Minimal feature set (no bloat)
- Clear separation of concerns
- No business logic (only I/O and initialization)

### 2. The GPU Compute Kernel (Sovereign Controller)

**File**: `shaders/editor_compute.wgsl`
**Language**: WGSL (WebGPU Shading Language)
**Execution Model**: Non-Stop Kernel (NSK)

#### The Non-Stop Kernel Pattern

Traditional GPU compute:
```
for each frame:
    CPU: dispatch_compute()
    GPU: execute_kernel()
    GPU: return_results()
```

GVPIE NSK:
```
CPU: dispatch_compute() once
GPU: while(running) {
    process_events()
    update_state()
    // never returns
}
```

#### Benefits of NSK

1. **Zero Dispatch Overhead**: Kernel runs continuously
2. **GPU Autonomy**: No CPU in the control loop
3. **Persistent State**: Memory survives across frames
4. **Event-Driven**: Reactive to input events

#### Memory Model

The compute kernel manages several persistent buffers:

**State Buffer** (1KB):
```wgsl
struct EditorState {
    cursor_line: atomic<u32>,      // Current cursor line
    cursor_col: atomic<u32>,       // Current cursor column
    scroll_line: atomic<u32>,      // Top visible line
    scroll_col: atomic<u32>,       // Left visible column
    text_length: atomic<u32>,      // Total characters
    line_count: atomic<u32>,       // Total lines
    key_ring_head: atomic<u32>,    // Input queue head
    key_ring_tail: atomic<u32>,    // Input queue tail
    running: atomic<u32>,          // Shutdown flag
    dirty: atomic<u32>,            // Needs recount
    frame_count: atomic<u32>,      // For animations
}
```

**Text Buffer** (40MB):
```wgsl
var<storage, read_write> text: array<u32>; // UTF-32 characters
```

**Key Ring Buffer** (1KB):
```wgsl
struct KeyEvent {
    scancode: u32,   // Virtual key code
    state: u32,      // Pressed/Released
    modifiers: u32,  // Shift/Ctrl/Alt
}

var<storage, read_write> key_ring: array<KeyEvent>; // Circular buffer
```

#### Synchronization Strategy

All state updates use atomic operations to ensure thread-safety:

```wgsl
// Thread-safe cursor movement
atomicStore(&state.cursor_col, new_col);
storageBarrier(); // Ensure visibility
```

Storage barriers ensure that:
1. Compute writes are visible to subsequent compute dispatches
2. Compute writes are visible to render shader
3. Memory ordering is preserved

#### Editor Logic Implementation

The compute kernel implements all editor functionality:

**Text Insertion**:
```wgsl
fn insert_char(c: u32) {
    let offset = cursor_to_offset();
    let len = atomicLoad(&state.text_length);
    
    // Shift text right
    for (var i = len; i > offset; i -= 1u) {
        text[i] = text[i - 1u];
    }
    
    text[offset] = c;
    atomicAdd(&state.text_length, 1u);
    advance_cursor();
    atomicStore(&state.dirty, 1u);
    storageBarrier();
}
```

**Cursor Movement**:
```wgsl
fn move_cursor_left() {
    let col = atomicLoad(&state.cursor_col);
    if col > 0u {
        atomicStore(&state.cursor_col, col - 1u);
    } else {
        // Wrap to previous line
        let line = atomicLoad(&state.cursor_line);
        if line > 0u {
            atomicStore(&state.cursor_line, line - 1u);
            atomicStore(&state.cursor_col, get_line_length(line - 1u));
        }
    }
}
```

### 3. The GPU Render Pipeline (Display Layer)

**File**: `shaders/editor_render.wgsl`
**Language**: WGSL
**Execution Model**: Per-frame fragment shader

#### Responsibilities

1. **Text Rendering**: Convert UTF-32 → screen pixels via font atlas
2. **Cursor Display**: Blinking cursor at current position
3. **Line Numbers**: Gutter with line numbering
4. **Scrolling**: Viewport management

#### Rendering Pipeline

**Vertex Shader**:
```wgsl
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // Generate fullscreen quad (two triangles)
    // Pass UV coordinates to fragment shader
}
```

**Fragment Shader**:
```wgsl
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Convert UV to character grid position
    let char_col = u32(uv.x * COLS_VISIBLE);
    let char_row = u32(uv.y * ROWS_VISIBLE);
    
    // 2. Apply scrolling
    let buffer_line = char_row + scroll_line;
    let buffer_col = char_col + scroll_col;
    
    // 3. Render line numbers or text
    if char_col < 4 {
        return render_line_number(buffer_line);
    }
    
    // 4. Check for cursor position
    if at_cursor_position(buffer_line, buffer_col) {
        return render_cursor();
    }
    
    // 5. Get character from text buffer
    let c = get_char_at(buffer_line, buffer_col);
    
    // 6. Render glyph via font atlas
    return render_glyph(c, pixel_x, pixel_y);
}
```

#### Font Atlas Format

Simple 8x8 monospace bitmap:
```
Character 'A' (ASCII 65):
  00111100  →  ░░▓▓▓▓░░
  01100110  →  ░▓▓░░▓▓░
  01100110  →  ░▓▓░░▓▓░
  01111110  →  ░▓▓▓▓▓▓░
  01100110  →  ░▓▓░░▓▓░
  01100110  →  ░▓▓░░▓▓░
  01100110  →  ░▓▓░░▓▓░
  00000000  →  ░░░░░░░░
```

95 characters (ASCII 32-126) = 760 bytes total.

## Data Flow Architecture

### CPU → GPU Communication

```
User Input
    ↓
OS Event (winit)
    ↓
Serialize to KeyEvent
    ↓
queue.write_buffer(key_ring_buffer)
    ↓
GPU Ring Buffer
    ↓
Compute Shader Reads
    ↓
Process Input & Update State
```

### GPU → CPU Communication

```
User Command (Ctrl+S)
    ↓
Set "save_requested" flag in state
    ↓
CPU polls state buffer
    ↓
Read text buffer back to CPU
    ↓
Write to file system
```

### GPU → GPU Communication (Compute ↔ Render)

```
Compute Pass:
    Update text buffer
    Update cursor position
    storageBarrier()
    ↓
Render Pass:
    Read text buffer (read-only)
    Read cursor position
    Render to screen
```

## Concurrency Model

### Thread Safety

All shared state uses atomic operations:

```wgsl
// Safe concurrent reads
let value = atomicLoad(&state.cursor_col);

// Safe concurrent writes
atomicStore(&state.cursor_col, new_value);

// Safe concurrent modifications
atomicAdd(&state.text_length, 1u);
```

### Memory Barriers

Three types of barriers ensure correctness:

1. **storageBarrier()**: Within compute shader, ensures writes are visible
2. **workgroupBarrier()**: Synchronize within workgroup (not used yet)
3. **Device synchronization**: CPU queue submission boundaries

### Race Condition Prevention

The single-threaded compute dispatch (workgroup_size = 1) eliminates most races. Future multi-VM support will require:

- Per-VM memory isolation
- Atomic resource allocation
- Fine-grained locking

## Performance Characteristics

### Bottlenecks (Current)

1. **Line Scanning**: O(n) search for line starts
2. **Text Shifting**: O(n) for insert/delete
3. **Frame Dispatches**: One compute + one render per frame
4. **Font Rendering**: Bitmap lookup per pixel

### Optimizations (Planned)

1. **Line Index Cache**: Precompute line start offsets
2. **Gap Buffer**: Reduce text shifting overhead
3. **Batched Compute**: Process multiple frames of input at once
4. **SDF Font**: Scalable distance field font rendering

### Theoretical Limits

With current buffer sizes:
- **Max File Size**: 10 million characters (~10MB UTF-32)
- **Max Lines**: ~100,000 lines (1KB state buffer)
- **Input Latency**: 1-2 frames (16-32ms at 60fps)
- **Memory Bandwidth**: Limited by GPU VRAM (100+ GB/s)

## Extensibility Model

### Adding Features (Week 2+)

**Example: Syntax Highlighting**

1. Add color array to state:
```wgsl
@group(0) @binding(4) var<storage, read_write> colors: array<u32>;
```

2. Implement tokenizer in compute shader:
```wgsl
fn tokenize_line(line: u32) {
    // Parse line for keywords
    // Store color codes in colors array
}
```

3. Render with colors:
```wgsl
let color_code = colors[char_index];
let color = decode_color(color_code);
return vec4<f32>(color, 1.0);
```

### Adding New Tools

**Example: GPU Terminal**

```wgsl
// New compute shader: terminal.wgsl
struct TerminalState {
    output_buffer: array<u32>,
    command_buffer: array<u32>,
    history: array<u32>,
}

@compute
fn terminal_main() {
    // Read command input
    // Execute (call into other shaders?)
    // Write to output buffer
}
```

## Security Model

### Isolation Boundaries

1. **CPU/GPU Boundary**: Only explicit buffer copies cross
2. **Compute/Render Boundary**: Read-only access from render
3. **Future VM Boundary**: Separate storage buffers per VM

### Privilege Separation

Current model:
```
Ring 0: CPU Kernel (OS)
Ring 1: CPU User (Rust bootstrap)
Ring X: GPU Compute (Editor kernel)
```

Target model (GPU-first):
```
EL2: GPU Hypervisor
EL1: GPU Guest Kernels
EL0: GPU Applications
(CPU relegated to peripheral controller)
```

### Attack Surface

**Minimal**: Only attack vectors are:
1. Malicious WGSL shader (requires file system access)
2. Buffer overflow in text buffer (bounded by allocation)
3. GPU driver vulnerabilities (outside our control)

## Future Roadmap

### Week 2: Self-Hosting Compiler

**Goal**: WGSL → SPIR-V compiler in WGSL

Components:
- Lexer (compute shader)
- Parser (compute shader)
- Semantic analysis (compute shader)
- Code generator (compute shader)
- Linker (compute shader)

### Week 3: GPU File System

**Goal**: Virtual FS in GPU storage buffers

```wgsl
struct FileSystem {
    inodes: array<Inode>,
    data_blocks: array<Block>,
    free_bitmap: array<u32>,
}

struct Inode {
    name: array<u32, 64>,
    size: u32,
    blocks: array<u32, 128>,
}
```

### Week 4: Hypervisor Core

**Goal**: VM scheduler and isolation

```wgsl
@compute @workgroup_size(1)
fn vm_scheduler(@builtin(global_invocation_id) vm_id: vec3<u32>) {
    // Each workgroup is a VM
    let vm = vms[vm_id.x];
    
    // Execute VM instructions
    while !vm.halted {
        execute_instruction(&vm);
        
        // Check for hypercalls
        if vm.hypercall_pending {
            handle_hypercall(&vm);
        }
    }
}
```

### Long-Term: Full GPU OS

Components:
- Process scheduler
- Memory manager
- Device drivers (GPU-resident)
- Network stack
- Storage controller

All written in WGSL, running on GPU.

## Hardware Requirements

### Minimum Viable

- GPU with WebGPU support (Vulkan 1.1+, Metal 2+, DX12)
- 2GB VRAM
- Support for storage buffers ≥128MB
- Support for compute shaders

### Optimal

- Unified Memory Architecture (AMD APU, Apple Silicon)
- 8GB+ unified memory
- Support for storage buffers ≥2GB
- Hardware-accelerated atomics
- Low-latency GPU dispatch

### Future (Required for Full Sovereignty)

- GPU Exception Levels (EL1/EL2 equivalent)
- Fine-grained preemption
- IOMMU control from GPU
- Direct PCIe access
- Interrupt handling from GPU

## Conclusion

This architecture proves that GPU-sovereign computing is feasible. By inverting the traditional control hierarchy and freezing the CPU layer, we've created a foundation for:

1. **Self-hosting development** (edit the editor in the editor)
2. **GPU-native toolchains** (compilers, linkers, debuggers in WGSL)
3. **True hypervisors** (VM management on GPU)
4. **Full operating systems** (GPU as primary controller)

The CPU is no longer the master. The GPU is sovereign.

---

**Document Version**: 1.0
**Last Updated**: October 2025
**Author**: GVPIE Bootstrap Team
