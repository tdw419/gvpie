struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var present_texture: texture_2d<f32>;

@group(0) @binding(1)
var present_sampler: sampler; // kept for compatibility, currently unused

struct OverlayUniform {
    cursor: vec4<f32>,          // (x, y, blink, text_origin_y)
    selection: vec4<f32>,       // (start, end, line_wrap, text_length)
    glyph_viewport: vec4<f32>,  // (glyph_width, glyph_height, viewport_width, viewport_height)
}

@group(1) @binding(0)
var<uniform> overlay: OverlayUniform;

@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    let pos = positions[vid];
    var out: VertexOutput;
    out.position = vec4<f32>(pos, 0.0, 1.0);
    out.uv = pos * 0.5 + vec2<f32>(0.5);
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let viewport = vec2<f32>(overlay.glyph_viewport.z, overlay.glyph_viewport.w);
    if (viewport.x == 0.0 || viewport.y == 0.0) {
        return vec4<f32>(0.0);
    }

    let tex_uv = vec2<f32>(input.uv.x, 1.0 - input.uv.y);
    let base_coord = vec2<i32>(floor(tex_uv * viewport));
    let dims = vec2<i32>(i32(max(viewport.x - 1.0, 0.0)), i32(max(viewport.y - 1.0, 0.0)));
    let coords = clamp(base_coord, vec2<i32>(0), dims);
    let sampled = textureLoad(present_texture, coords, 0);
    var color = vec4<f32>(sampled.rgb, 1.0);

    let pixel = vec2<f32>(coords);
    let glyph_width = overlay.glyph_viewport.x;
    let glyph_height = overlay.glyph_viewport.y;

    let cursor_pos = overlay.cursor.xy;
    let blink = overlay.cursor.z;
    let text_origin_y = overlay.cursor.w;

    if (blink > 0.5) {
        if (pixel.x >= cursor_pos.x && pixel.x < cursor_pos.x + glyph_width &&
            pixel.y >= cursor_pos.y && pixel.y < cursor_pos.y + glyph_height) {
            color = mix(color, vec4<f32>(1.0, 1.0, 0.0, 1.0), 0.7);
        }
    }

    let selection_start = u32(round(overlay.selection.x));
    let selection_end = u32(round(overlay.selection.y));
    let line_wrap = u32(round(overlay.selection.z));
    let text_length = u32(round(overlay.selection.w));

    if (selection_end > selection_start && line_wrap > 0u && glyph_width > 0.0 && glyph_height > 0.0) {
        if (pixel.y >= text_origin_y) {
            let rel_y = pixel.y - text_origin_y;
            if (rel_y >= 0.0) {
                let line = u32(floor(rel_y / glyph_height));
                let col = u32(floor(pixel.x / glyph_width));
                let idx = line * line_wrap + col;
                if (idx >= selection_start && idx < selection_end && idx < text_length) {
                    color = mix(color, vec4<f32>(0.2, 0.4, 1.0, 1.0), 0.3);
                }
            }
        }
    }

    return color;
}
