struct HexConfig {
    origin_x : u32,
    origin_y : u32,
    bytes_per_row : u32,
    row_count : u32,
    stride : u32,
    _pad0 : u32,
    _pad1 : u32,
    _pad2 : u32,
};

@group(0) @binding(0) var work_tex : texture_2d<u32>;
@group(0) @binding(1) var<storage, read_write> hex_chars : array<u32>;
@group(0) @binding(2) var<uniform> hex_cfg : HexConfig;

fn nibble_to_ascii(nibble : u32) -> u32 {
    let base_num = 48u + nibble;
    let base_alpha = 55u + nibble;
    return select(base_num, base_alpha, nibble > 9u);
}

@compute @workgroup_size(16, 1, 1)
fn hex_formatter(@builtin(global_invocation_id) id : vec3<u32>) {
    let row = id.y;
    let col = id.x;

    if (row >= hex_cfg.row_count || col >= hex_cfg.bytes_per_row) {
        return;
    }

    let coord = vec2<i32>(i32(hex_cfg.origin_x + col), i32(hex_cfg.origin_y + row));
    let texel = textureLoad(work_tex, coord, 0);
    let byte_value = texel.r & 0xFFu;

    let hi = (byte_value >> 4u) & 0xFu;
    let lo = byte_value & 0xFu;

    let ascii_hi = nibble_to_ascii(hi);
    let ascii_lo = nibble_to_ascii(lo);

    let chars_per_row = hex_cfg.bytes_per_row * 3u;
    let base = row * chars_per_row + col * 3u;

    hex_chars[base + 0u] = ascii_hi;
    hex_chars[base + 1u] = ascii_lo;
    hex_chars[base + 2u] = 32u;
}
