# GVPIE I/O Contract v1.0

## Overview

The I/O contract is the frozen interface between the CPU bootstrap and the GPU runtime. It consists of three shared buffers:

- **Events Buffer (Host → GPU)** – keyboard, mouse, and window events marshalled each frame.
- **Request Buffer (GPU → Host)** – GPU-originated requests for file I/O, shader hot reload, exit, etc.
- **File I/O Buffer (Bidirectional)** – 1 MiB scratch buffer used to transfer file contents without copies.

The structures that define these buffers live in `src/io_contract.rs` and are **immutable after the freeze**. Future evolution must use the `version` field for compatibility.

## Layout

```
GPU Memory (contract region)
┌───────────────────────────────────────────────┐
│ EventsBuffer (≈4 KB)                          │ ← CPU writes, GPU reads
│  - version, event_count, frame_number         │
│  - events[256] → { event_type, data0..2 }     │
├───────────────────────────────────────────────┤
│ RequestBuffer (≈1 KB)                         │ ← GPU writes, CPU reads
│  - version, request_count, request_id_counter │
│  - requests[16] → { type, status, params, path } │
├───────────────────────────────────────────────┤
│ FileIOBuffer (1,048,576 bytes)                │ ← Shared data window
│  - raw byte storage                           │
└───────────────────────────────────────────────┘
```

Sizes are computed in `io_contract::buffer_sizes` for the bootstrap to allocate exact GPU buffers.

## Event Semantics

`EventType`

- `KeyPress` / `KeyRelease`: `data0 = scancode`, `data1 = unicode`, `data2 = modifiers`.
- `MouseMove`: `data0 = Δx`, `data1 = Δy` (future use).
- `MousePress` / `MouseRelease`: `data0 = button id`.
- `WindowResize`: `data0 = width`, `data1 = height`.
- `Scroll`: `data0 = Δx`, `data1 = Δy` (future use).

The bootstrap increments `frame_number`, writes the buffer, then clears `event_count` every frame.

## Request Semantics

`RequestType`

- `FileRead`: Host reads file at `path`, writes bytes into `FileIOBuffer`, sets `params[0]=bytes_read`, marks `status=Success` (or `Error`).
- `FileWrite`: Host writes `params[0]` bytes from `FileIOBuffer` to `path` on disk.
- `DirList`: Future extension (reserved).
- `ShaderReload`: Host reloads shader indicated by `params[0]` when in dev mode.
- `Exit`: Host terminates application gracefully.

Requests remain `Pending` until the host updates `status`.

## WGSL Usage

```wgsl
struct HostEvent {
    event_type: u32;
    data0: u32;
    data1: u32;
    data2: u32;
};

struct EventsBuffer {
    version: u32;
    event_count: u32;
    frame_number: u32;
    _padding: u32;
    events: array<HostEvent, 256>;
};

@group(0) @binding(4)
var<storage, read> host_events: EventsBuffer;

fn process_events() {
    let count = min(host_events.event_count, 256u);
    var i = 0u;
    loop {
        if i >= count { break; }
        let event = host_events.events[i];
        if event.event_type == 1u { /* KeyPress handling */ }
        i = i + 1u;
    }
}
```

Requests use the analogous `RequestBuffer` structure when the GPU needs host services.

## Guarantees

- The contract is **versioned** (current `version = 1`).
- Structures are `#[repr(C)]` in Rust and mirrored exactly in WGSL.
- All buffers are zeroed on bootstrap startup.
- The CPU never mutates GPU state outside these buffers and shader reloads.

Any future capability must either:

1. Build on top of these buffers without altering their layouts, or
2. Introduce a new contract version that the GPU explicitly negotiates.

The freeze makes the GPU sovereign: once the bootstrap is frozen, only GPU shaders evolve.
```
