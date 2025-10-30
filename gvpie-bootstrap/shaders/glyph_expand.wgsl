@group(0) @binding(0) var machine_texture: texture_2d<u32>;
@group(0) @binding(1) var human_texture: texture_storage_2d<rgba8unorm, write>;

const GLYPH_WIDTH: i32 = 5;
const GLYPH_HEIGHT: i32 = 7;
const GLYPH_WIDTH_U: u32 = 5u;
const GLYPH_HEIGHT_U: u32 = 7u;
const FIRST_PRINTABLE: u32 = 32u;
const LAST_PRINTABLE: u32 = 126u;

fn color_to_char(color: vec3<u32>) -> u32 {
    return color.r;
}

fn atlas_pixel(char_code: u32, gx: u32, gy: u32) -> u32 {
    let atlas_x = i32((char_code - FIRST_PRINTABLE) * GLYPH_WIDTH_U + gx);
    let atlas_coord = vec2<i32>(atlas_x, i32(gy));
    let atlas_texel = textureLoad(machine_texture, atlas_coord, 0).r;
    return select(0u, 1u, atlas_texel > 127u);
}

@compute @workgroup_size(8, 8)
fn expand_glyphs(@builtin(global_invocation_id) id: vec3<u32>) {
    let dims = textureDimensions(machine_texture);
    if (id.x >= dims.x || id.y >= dims.y) {
        return;
    }

    let machine_coord = vec2<i32>(i32(id.x), i32(id.y));
   let texel = textureLoad(machine_texture, machine_coord, 0);
    let char_code = color_to_char(texel.rgb);

    if (char_code < FIRST_PRINTABLE || char_code > LAST_PRINTABLE) {
        return;
    }

    for (var gy: i32 = 0; gy < GLYPH_HEIGHT; gy = gy + 1) {
        for (var gx: i32 = 0; gx < GLYPH_WIDTH; gx = gx + 1) {
            let pixel = atlas_pixel(char_code, u32(gx), u32(gy));
            let human_coord = machine_coord * vec2<i32>(GLYPH_WIDTH, GLYPH_HEIGHT) + vec2<i32>(gx, gy);
            let color = select(
                vec4<f32>(0.0, 0.0, 0.0, 1.0),
                vec4<f32>(1.0, 1.0, 1.0, 1.0),
                pixel != 0u,
            );
            textureStore(human_texture, human_coord, color);
        }
    }

    // Debug: write a known white pixel to verify human texture writes
    if (id.x == 0u && id.y == 0u) {
        textureStore(human_texture, vec2<i32>(2, 4903), vec4<f32>(1.0, 1.0, 1.0, 1.0));
    }
}
