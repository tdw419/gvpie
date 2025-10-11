// GVPIE Editor Render Shader
// Displays the text buffer as a terminal-style editor

// ============================================================================
// SHARED STATE (Read-Only for Rendering)
// ============================================================================

struct EditorState {
    cursor_line: atomic<u32>,
    cursor_col: atomic<u32>,
    scroll_line: atomic<u32>,
    scroll_col: atomic<u32>,
    text_length: atomic<u32>,
    line_count: atomic<u32>,
    key_ring_head: atomic<u32>,
    key_ring_tail: atomic<u32>,
    running: atomic<u32>,
    dirty: atomic<u32>,
    frame_count: atomic<u32>,
    reserved: array<u32, 245>,
}

@group(0) @binding(0) var<storage, read> state: EditorState;
@group(0) @binding(1) var<storage, read> text: array<u32>;
@group(0) @binding(3) var<storage, read> font_atlas: array<u32>; // 8x8 bitmap font

// ============================================================================
// CONSTANTS
// ============================================================================

const CHAR_WIDTH: f32 = 8.0;
const CHAR_HEIGHT: f32 = 16.0;
const COLS_VISIBLE: u32 = 150u;
const ROWS_VISIBLE: u32 = 50u;

const COLOR_BG: vec3<f32> = vec3<f32>(0.05, 0.06, 0.08);
const COLOR_FG: vec3<f32> = vec3<f32>(0.9, 0.9, 0.9);
const COLOR_CURSOR: vec3<f32> = vec3<f32>(0.2, 0.8, 0.3);
const COLOR_LINE_NUM: vec3<f32> = vec3<f32>(0.4, 0.4, 0.5);

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn is_newline(c: u32) -> bool {
    return c == 10u;
}

fn get_line_start(line: u32) -> u32 {
    if line == 0u { return 0u; }
    
    var count: u32 = 0u;
    var pos: u32 = 0u;
    let len = atomicLoad(&state.text_length);
    
    while pos < len && count < line {
        if is_newline(text[pos]) {
            count += 1u;
        }
        pos += 1u;
    }
    
    return pos;
}

fn get_char_at(line: u32, col: u32) -> u32 {
    let start = get_line_start(line);
    let len = atomicLoad(&state.text_length);
    let pos = start + col;
    
    if pos >= len {
        return 0u; // Out of bounds
    }
    
    let c = text[pos];
    if is_newline(c) {
        return 0u; // Don't render newlines
    }
    
    return c;
}

fn render_glyph(c: u32, pixel_x: u32, pixel_y: u32) -> f32 {
    // Simple ASCII rendering (32-126)
    if c < 32u || c > 126u {
        return 0.0;
    }
    
    let glyph_index = c - 32u;
    let font_offset = glyph_index * 8u; // 8 bytes per character
    
    if font_offset >= 760u {
        return 0.0; // Out of font atlas bounds
    }
    
    let row = font_atlas[font_offset + pixel_y];
    let bit = (row >> (7u - pixel_x)) & 1u;
    
    return f32(bit);
}

fn draw_number(num: u32, x: u32, y: u32, pixel_x: u32, pixel_y: u32) -> f32 {
    // Draw a number digit (0-9)
    let digit = (num / u32(pow(10.0, f32(2u - x)))) % 10u;
    let c = 48u + digit; // ASCII '0' + digit
    return render_glyph(c, pixel_x, pixel_y);
}

// ============================================================================
// VERTEX SHADER (Fullscreen Quad)
// ============================================================================

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_idx: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
    );
    
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );
    
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_idx], 0.0, 1.0);
    output.uv = uvs[vertex_idx];
    return output;
}

// ============================================================================
// FRAGMENT SHADER (Text Rendering)
// ============================================================================

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Convert UV to screen coordinates
    let screen_x = input.uv.x * CHAR_WIDTH * f32(COLS_VISIBLE);
    let screen_y = input.uv.y * CHAR_HEIGHT * f32(ROWS_VISIBLE);
    
    // Convert to character grid coordinates
    let char_col = u32(floor(screen_x / CHAR_WIDTH));
    let char_row = u32(floor(screen_y / CHAR_HEIGHT));
    
    // Convert to pixel within character
    let pixel_x = u32(screen_x) % u32(CHAR_WIDTH);
    let pixel_y = u32(screen_y) % u32(CHAR_HEIGHT);
    
    // Get scroll offset
    let scroll_line = atomicLoad(&state.scroll_line);
    let scroll_col = atomicLoad(&state.scroll_col);
    
    // Adjust for scrolling
    let buffer_line = char_row + scroll_line;
    let buffer_col = char_col + scroll_col;
    
    // Line number gutter (first 4 columns)
    if char_col < 4u {
        let intensity = draw_number(buffer_line + 1u, char_col, char_row, pixel_x, pixel_y);
        return vec4<f32>(COLOR_LINE_NUM * intensity, 1.0);
    }
    
    // Adjust for gutter
    let text_col = buffer_col - 4u;
    
    // Check if we're at cursor position
    let cursor_line = atomicLoad(&state.cursor_line);
    let cursor_col = atomicLoad(&state.cursor_col);
    
    if buffer_line == cursor_line && text_col == cursor_col {
        // Blinking cursor
        let frame = atomicLoad(&state.frame_count);
        let blink = (frame / 30u) % 2u; // Blink every 30 frames
        if blink == 1u {
            return vec4<f32>(COLOR_CURSOR, 1.0);
        }
    }
    
    // Get character at this position
    let c = get_char_at(buffer_line, text_col);
    
    if c == 0u {
        return vec4<f32>(COLOR_BG, 1.0);
    }
    
    // Render the glyph
    let intensity = render_glyph(c, pixel_x, pixel_y);
    let color = mix(COLOR_BG, COLOR_FG, intensity);
    
    return vec4<f32>(color, 1.0);
}
