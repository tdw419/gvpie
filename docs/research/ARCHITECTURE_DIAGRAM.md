# GVPIE Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GVPIE GPU-NATIVE COMPUTING STACK                 │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                          USER INTERACTION                           │
│                                                                     │
│  Keyboard → Window Events → OS Event Queue → User Input            │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ↓
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 1: FROZEN CPU BOOTSTRAP                    │
│                       (src/main.rs - 400 lines)                     │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐            │
│  │   Window     │  │     GPU      │  │   Event      │            │
│  │ Management   │  │    Init      │  │  Marshal     │            │
│  │  (winit)     │  │   (wgpu)     │  │   (I/O)      │            │
│  └──────────────┘  └──────────────┘  └──────────────┘            │
│                                                                     │
│  Role: Initialize once, then freeze                                │
│  Never modified after Day 2                                        │
│  Only handles: Window, GPU context, File I/O                       │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ↓ (Events to GPU Ring Buffer)
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 2: GPU COMPUTE KERNEL                      │
│              (shaders/editor_compute.wgsl - 400 lines)              │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────┐    │
│  │          NON-STOP KERNEL (Runs Forever)                   │    │
│  │                                                            │    │
│  │  @compute @workgroup_size(1)                              │    │
│  │  fn main() {                                              │    │
│  │      initialize_if_first_run();                           │    │
│  │                                                            │    │
│  │      while running {                                      │    │
│  │          process_input_queue();    ← Read from ring       │    │
│  │          update_text_buffer();     ← Modify text          │    │
│  │          move_cursor();            ← Update position      │    │
│  │          recalculate_lines();      ← Update metadata      │    │
│  │          storageBarrier();         ← Sync memory          │    │
│  │      }                                                     │    │
│  │  }                                                         │    │
│  └───────────────────────────────────────────────────────────┘    │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              GPU PERSISTENT MEMORY                           │ │
│  │                                                              │ │
│  │  State Buffer (1KB):                                        │ │
│  │  ├─ cursor_line, cursor_col                                │ │
│  │  ├─ text_length, line_count                                │ │
│  │  ├─ key_ring_head, key_ring_tail                           │ │
│  │  └─ frame_count, dirty flags                               │ │
│  │                                                              │ │
│  │  Text Buffer (40MB):                                        │ │
│  │  └─ UTF-32 character array                                 │ │
│  │                                                              │ │
│  │  Key Ring (1KB):                                            │ │
│  │  └─ Circular buffer of keyboard events                     │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  Role: ALL editor logic                                            │
│  This is where development happens                                 │
│  Modify this to change behavior                                    │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ↓ (Memory visible to render)
┌─────────────────────────────────────────────────────────────────────┐
│                    LAYER 3: GPU RENDER PIPELINE                     │
│              (shaders/editor_render.wgsl - 200 lines)               │
│                                                                     │
│  @vertex fn vs_main() → Fullscreen quad                            │
│                                                                     │
│  @fragment fn fs_main() {                                          │
│      Read text_buffer (read-only)                                  │
│      Read cursor_position (read-only)                              │
│      Render characters via font_atlas                              │
│      Display cursor (blinking)                                     │
│      Show line numbers                                             │
│      Apply scrolling                                               │
│  }                                                                  │
│                                                                     │
│  Role: Pure display                                                │
│  No logic, just visualization                                      │
│  Reads GPU state, outputs pixels                                   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ↓
┌─────────────────────────────────────────────────────────────────────┐
│                          SCREEN OUTPUT                              │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────┐      │
│  │ GVPIE Editor                                     [×]    │      │
│  ├─────────────────────────────────────────────────────────┤      │
│  │ 001  GVPIE Editor                                       │      │
│  │ 002  GPU-Native Development                             │      │
│  │ 003                                                      │      │
│  │ 004  Type here... █                                     │      │
│  │ 005                                                      │      │
│  │ ...                                                      │      │
│  └─────────────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════════

                        CONTROL FLOW INVERSION

Traditional:                    GVPIE:
                               
CPU (Master)                   CPU (Frozen)
    ↓ controls                     ↓ initializes
GPU (Servant)                  GPU (Sovereign)
                                   ↓ controls
                               Everything

═══════════════════════════════════════════════════════════════════════

                         DATA FLOW SUMMARY

Input Events:
  Keyboard → CPU → Ring Buffer → GPU Compute → Process

State Updates:
  GPU Compute → Storage Buffers → GPU Compute (next frame)

Rendering:
  Storage Buffers → GPU Render → Screen Pixels

File I/O:
  GPU → CPU (save command) → Disk
  Disk → CPU (load command) → GPU

═══════════════════════════════════════════════════════════════════════

                    SELF-HOSTING DEVELOPMENT LOOP

  ┌─────────────────────────────────────────────────┐
  │                                                  │
  ↓                                                  │
Edit editor_compute.wgsl                            │
  ↓                                                  │
Save file (CPU writes to disk)                      │
  ↓                                                  │
Hot-reload shader                                   │
  ↓                                                  │
GPU recompiles and loads new behavior              │
  ↓                                                  │
Editor now operates with new logic                 │
  ↓                                                  │
Use updated editor to make more improvements ───────┘

═══════════════════════════════════════════════════════════════════════

                      KEY ARCHITECTURAL PROPERTIES

✓ CPU Bootstrap: Frozen after Day 2
✓ GPU Sovereignty: All logic on GPU
✓ Persistent State: Survives across frames
✓ Event-Driven: Reactive to input
✓ Self-Hosting: Editor edits itself
✓ Zero Dispatch Overhead: Non-Stop Kernel
✓ Memory Safety: Atomic operations
✓ Scalable: Foundation for OS stack

═══════════════════════════════════════════════════════════════════════
```
