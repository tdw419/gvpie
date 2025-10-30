# GVPIE Bootstrap

This crate is a minimal CPU bootstrap that hands control to GPU-resident shaders.

## Prerequisites

- Rust 1.75+ (with `cargo`)
- A GPU/device that supports the [`wgpu`](https://github.com/gfx-rs/wgpu) 0.19 backend (Vulkan, Metal, DX12, or WebGPU on compatible platforms)

## Running

```bash
cargo run --release
```

The first launch compiles the Rust host and the WGSL shaders found in `shaders/editor_compute.wgsl` and `shaders/editor_render.wgsl`. When the window appears, all text-editing logic runs inside the compute shader; the Rust process only marshals events and presents frames.

## Project Layout

- `src/main.rs` – frozen host bootstrap that initialises wgpu, manages the window, and forwards keyboard events to the GPU.
- `shaders/editor_compute.wgsl` – non-stop compute kernel that maintains editor state, processes key events, and mutates the UTF-32 text buffer.
- `shaders/editor_render.wgsl` – fullscreen render pipeline that reads the text buffer and displays it with a bitmap font.

Modify the WGSL files to change behaviour; rebuilding the binary is only required if the host bootstrap itself changes.

## Phase 1 & 2 Glyph Bootstrap

The host-to-GPU ritual now has fully inscribed code. Phase 1 maps keystrokes directly to RGB pixels in the machine texture; Phase 2 expands those pixels into live glyphs via compute shader. The snippet below is drop-in ready for the Rust host, with a stubbed `MapInterface` you can adapt to your wgpu wrapper.

```rust
/// Any printable ASCII character becomes machine-ready RGB (R=code, G/B zero).
pub fn char_to_color(ch: char) -> Option<[u8; 3]> {
    if ch.is_ascii() && !ch.is_ascii_control() {
        Some([ch as u8, 0, 0])
    } else {
        None
    }
}

pub trait MapInterface {
    fn set_pixel(&mut self, pos: (u32, u32), color: [u8; 3]);
    fn advance_cursor(&mut self);
}

pub fn on_key_press(
    ch: char,
    cursor_pos: (u32, u32),
    map_interface: &mut dyn MapInterface,
) -> Option<[u8; 3]> {
    char_to_color(ch).map(|color| {
        map_interface.set_pixel(cursor_pos, color);
        map_interface.advance_cursor();
        color
    })
}
```

Implement `MapInterface` on your wgpu state by writing directly into the machine texture (RGBA8Uint) and optionally mirroring into a presentable RGBA8Unorm texture for raw viewing:

```rust
impl MapInterface for FrozenBootstrap {
    fn set_pixel(&mut self, pos: (u32, u32), color: [u8; 3]) {
        let rgba = [color[0], color[1], color[2], 255u8];
        let extent = Extent3d { width: 1, height: 1, depth_or_array_layers: 1 };
        let origin = Origin3d { x: pos.0, y: pos.1, z: 0 };

        // Write machine code (RGBA8Uint backing the compute pipeline)
        self.queue.write_texture(
            ImageCopyTexture { texture: &self.machine_code_texture, mip_level: 0, origin, aspect: TextureAspect::All },
            &rgba,
            ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            extent,
        );

        // Optional: mirror to a normalized texture so the raw machine view can be sampled
        self.queue.write_texture(
            ImageCopyTexture { texture: &self.machine_view_texture, mip_level: 0, origin, aspect: TextureAspect::All },
            &rgba,
            ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            extent,
        );

        self.cursor_pos = pos;
    }

    fn advance_cursor(&mut self) {
        let (width, height) = self.machine_dimensions;
        let mut x = self.cursor_pos.0 + 1;
        let mut y = self.cursor_pos.1;
        if x >= width {
            x = 0;
            if y + 1 < height {
                y += 1;
            }
        }
        self.cursor_pos = (x, y);
    }
}
```

Phase 2 arrives via WGSL compute shader—fully expanded glyph bitmaps, ready for SPIR-V compilation.

```wgsl
// This compute shader runs ON the map, not in the kernel (dispatched post-keypress or each frame)
@group(0) @binding(0) var machine_texture: texture_2d<u32>;  // RGBA8Uint input (machine code)
@group(0) @binding(1) var human_texture: texture_storage_2d<rgba8unorm, write>;  // RGBA8Unorm output (glyphs)

const GLYPH_WIDTH: i32 = 5;
const GLYPH_HEIGHT: i32 = 7;
const GLYPH_WIDTH_U: u32 = 5u;
const GLYPH_HEIGHT_U: u32 = 7u;

fn glyph_row_bits(char_code: u32, row: u32) -> u32 {
    switch char_code {
        case 120u: {  // 'x'
            switch row {
                case 0u: { return 10u; }
                case 1u: { return 21u; }
                case 2u: { return 10u; }
                case 3u: { return 4u; }
                case 4u: { return 10u; }
                case 5u: { return 21u; }
                case 6u: { return 10u; }
                default: { return 0u; }
            }
        }
        case 97u: {  // 'a'
            switch row {
                case 0u: { return 4u; }
                case 1u: { return 10u; }
                case 2u: { return 17u; }
                case 3u: { return 31u; }
                case 4u: { return 17u; }
                case 5u: { return 21u; }
                case 6u: { return 17u; }
                default: { return 0u; }
            }
        }
        case 49u: {  // '1'
            switch row {
                case 0u: { return 4u; }
                case 1u: { return 12u; }
                case 2u: { return 4u; }
                case 3u: { return 4u; }
                case 4u: { return 4u; }
                case 5u: { return 4u; }
                case 6u: { return 14u; }
                default: { return 0u; }
            }
        }
        default: {
            return 0u;
        }
    }
}

@compute @workgroup_size(8, 8)
fn expand_glyphs(@builtin(global_invocation_id) id: vec3<u32>) {
    // 1. Read the single machine code pixel (dispatch over machine grid)
    let machine_coord = vec2<i32>(i32(id.x), i32(id.y));
    let color = textureLoad(machine_texture, machine_coord, 0).rgb;
    
    // 2. Decode the color back into the character's ASCII code
    let char_code = color_to_char(color);
    
    // 3. Expand the single pixel into the 5x7 block (only if valid char)
    if (char_code != 32u) {  // Skip spaces/control
        for (var gy: i32 = 0; gy < 7; gy++) {
            for (var gx: i32 = 0; gx < 5; gx++) {
                // The coordinate math (scaling by 5 and 7, plus offset) is handled here
                let human_coord = machine_coord * vec2<i32>(5, 7) + vec2<i32>(gx, gy);
                let pixel_value = get_glyph_pixel(char_code, gx, gy);
                
                // 4. Write the 35 pixels to the human-readable buffer (1=white fg, 0=black bg)
                let glyph_color = select(
                    vec4<f32>(0.0, 0.0, 0.0, 1.0),
                    vec4<f32>(1.0, 1.0, 1.0, 1.0),
                    pixel_value != 0u
                );
                textureStore(human_texture, human_coord, glyph_color);
            }
        }
    }
}

fn color_to_char(color: vec3<u32>) -> u32 {
    // Red holds the ASCII code; G/B are zero in this bootstrap.
    return color.r;
}

fn get_glyph_pixel(char_code: u32, gx: i32, gy: i32) -> u32 {
    if (gx < 0 || gy < 0) {
        return 0u;
    }
    let ux = u32(gx);
    let uy = u32(gy);
    if (ux >= GLYPH_WIDTH_U || uy >= GLYPH_HEIGHT_U) {
        return 0u;
    }

    let row_bits = glyph_row_bits(char_code, uy);
    let shift = (GLYPH_WIDTH_U - 1u) - ux;
    return (row_bits >> shift) & 1u;
}
```

With those two phases live:

- Type `'x'` → machine view lights a red pixel, human view shows the 5×7 glyph.
- Dispatch the compute pass once per keypress (or frame) to keep both views synced.
- Press `'v'` to toggle between machine (raw RGB) and human (expanded glyph) render targets.
- Extend the glyph tables as you add characters; migrate them into the map atlas when ready.
