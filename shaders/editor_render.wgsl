// Vertex shader generates a fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // Generate positions for a fullscreen triangle
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),  // Triangle 1
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),  // Triangle 2
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

struct EditorState {
    cursor_x: u32,
    cursor_y: u32,
    frame_width: u32,
    frame_height: u32,
    time: f32,
    flags: u32,
}

@group(0) @binding(0) var<storage, read_write> state: EditorState;
@group(0) @binding(1) var<storage, read_write> text_buffer: array<u32>;
@group(0) @binding(2) var<storage, read_write> key_ring: array<u32>;
@group(0) @binding(3) var<storage, read> font_atlas: array<u32>;

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    // Calculate normalized UV coordinates
    let uv = pos.xy / vec2<f32>(state.frame_width, state.frame_height);

    // Draw a simple test pattern
    let grid_size = 32.0;
    let grid = floor(pos.xy / grid_size);
    let checkerboard = (grid.x + grid.y) % 2.0;

    // Mix between dark and light based on checkerboard
    let dark = vec4<f32>(0.1, 0.1, 0.1, 1.0);
    let light = vec4<f32>(0.2, 0.2, 0.2, 1.0);
    let color = mix(dark, light, checkerboard);

    // Highlight cursor position
    let cursor_pos = vec2<f32>(f32(state.cursor_x), f32(state.cursor_y));
    let cursor_size = vec2<f32>(2.0, 16.0);
    let cursor_rect = abs(pos.xy - cursor_pos) < cursor_size;
    let is_cursor = cursor_rect.x && cursor_rect.y;

    return select(color, vec4<f32>(1.0, 1.0, 1.0, 1.0), is_cursor);
}