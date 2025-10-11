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
@group(0) @binding(4) var<storage, read> events_buffer: array<u32>;

@compute @workgroup_size(1)
fn main() {
    // Process input events and update state
    state.cursor_x = 100u;
    state.cursor_y = 100u;
    state.time = state.time + 0.016666667;  // ~60 FPS

    // Process text input (TODO)
    text_buffer[0] = 72u;  // 'H'
    text_buffer[1] = 101u; // 'e'
    text_buffer[2] = 108u; // 'l'
    text_buffer[3] = 108u; // 'l'
    text_buffer[4] = 111u; // 'o'
}