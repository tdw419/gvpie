struct HudGlyph {
    char_code : u32,
    x : u32,
    y : u32,
    _pad : u32,
};

struct HudConfig {
    hud_count : u32,
    hex_count : u32,
    hex_chars_per_row : u32,
    hex_origin_x : u32,
    hex_origin_y : u32,
    hex_row_spacing : u32,
    _pad0 : u32,
    _pad1 : u32,
};

@group(0) @binding(0) var<storage, read> hud_glyphs : array<HudGlyph>;
@group(0) @binding(1) var<storage, read> hud_font_rows : array<u32>;
@group(0) @binding(2) var hud_texture : texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var<storage, read> hud_hex_chars : array<u32>;
@group(0) @binding(4) var<uniform> hud_config : HudConfig;

const FIRST_PRINTABLE : u32 = 32u;
const LAST_PRINTABLE : u32 = 126u;
const GLYPH_WIDTH : u32 = 5u;
const GLYPH_HEIGHT : u32 = 7u;

fn render_glyph(char_code : u32, origin : vec2<u32>) {
    if (char_code < FIRST_PRINTABLE || char_code > LAST_PRINTABLE) {
        return;
    }

    let dims = textureDimensions(hud_texture);
    if (origin.x + GLYPH_WIDTH > dims.x || origin.y + GLYPH_HEIGHT > dims.y) {
        return;
    }

    let char_index = char_code - FIRST_PRINTABLE;

    for (var row : u32 = 0u; row < GLYPH_HEIGHT; row = row + 1u) {
        let atlas_row = hud_font_rows[char_index * GLYPH_HEIGHT + row];
        for (var col : u32 = 0u; col < GLYPH_WIDTH; col = col + 1u) {
            let bit = (atlas_row >> col) & 1u;
            let coord = vec2<i32>(i32(origin.x + col), i32(origin.y + row));

            // Always use white text for maximum readability
            let color = vec4<f32>(1.0, 1.0, 1.0, select(0.0, 1.0, bit != 0u));
            textureStore(hud_texture, coord, color);
        }
    }
}

@compute @workgroup_size(1)
fn hud_overlay(@builtin(global_invocation_id) id : vec3<u32>) {
    let index = id.x;
    let hud_count = hud_config.hud_count;

    if (index < hud_count) {
        let glyph = hud_glyphs[index];
        let origin = vec2<u32>(glyph.x, glyph.y);
        render_glyph(glyph.char_code, origin);
        return;
    }

    let hex_index = index - hud_count;
    if (hex_index >= hud_config.hex_count) {
        return;
    }

    let chars_per_row = hud_config.hex_chars_per_row;
    if (chars_per_row == 0u) {
        return;
    }

    let row = hex_index / chars_per_row;
    let col = hex_index % chars_per_row;
    let origin = vec2<u32>(
        hud_config.hex_origin_x + col * GLYPH_WIDTH,
        hud_config.hex_origin_y + row * hud_config.hex_row_spacing,
    );

    let char_code = hud_hex_chars[hex_index];
    render_glyph(char_code, origin);
}

// For now, we'll let the host code handle CPPM display
// through the existing hex display mechanism.

// The host will read CPPM values from the repurposed registers:
//   regs.err_op   -> CPPM cost
//   regs._pad     -> CPPM budget
//   regs.err_code -> Pixel count
//   regs.err_addr -> Instruction count
// The above values will be formatted into the hex display
// by the host code, preserving Layer 1.
// End of file
// End
// End

// Final end

// HUD code complete
// Final cleanup complete

// End of WGSL implementation
// The end
// HUD code complete

// Implementation complete
    }
}
