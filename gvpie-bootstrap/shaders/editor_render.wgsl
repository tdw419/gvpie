// GVPIE Editor Render Shader
// Renders the UTF-32 text buffer using a bitmap font

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
};

struct KeyEvent {
    scancode: u32,
    state: u32,
    modifiers: u32,
    _padding: u32,
};

@group(0) @binding(0) var<storage, read_write> state: EditorState;
@group(0) @binding(1) var<storage, read_write> text: array<u32>;
@group(0) @binding(2) var<storage, read_write> key_ring: array<KeyEvent>;
@group(0) @binding(3) var<storage, read> font_atlas: array<u32>;

const CHAR_WIDTH: f32 = 8.0;
const CHAR_HEIGHT: f32 = 8.0;
const COLS_VISIBLE: u32 = 160u;
const ROWS_VISIBLE: u32 = 90u;
const GUTTER_COLS: u32 = 4u;

const COLOR_BG: vec3<f32> = vec3<f32>(0.05, 0.06, 0.08);
const COLOR_FG: vec3<f32> = vec3<f32>(0.9, 0.9, 0.9);
const COLOR_GUTTER: vec3<f32> = vec3<f32>(0.35, 0.40, 0.45);
const COLOR_CURSOR: vec3<f32> = vec3<f32>(0.2, 0.8, 0.3);

fn is_newline(value: u32) -> bool {
    return value == 10u;
}

fn get_line_start(line_index: u32) -> u32 {
    if line_index == 0u {
        return 0u;
    }

    var count: u32 = 0u;
    var index: u32 = 0u;
    let len = atomicLoad(&state.text_length);

    loop {
        if index >= len || count == line_index {
            break;
        }

        if is_newline(text[index]) {
            count = count + 1u;
        }

        index = index + 1u;
    }

    return index;
}

fn get_char_at(line: u32, col: u32) -> u32 {
    let start = get_line_start(line);
    let len = atomicLoad(&state.text_length);
    let index = start + col;

    if index >= len {
        return 32u;
    }

    let value = text[index];
    if is_newline(value) {
        return 32u;
    }

    return value;
}

fn sample_glyph(codepoint: u32, cell_x: u32, cell_y: u32) -> f32 {
    if codepoint < 32u || codepoint > 126u {
        return 0.0;
    }

    let glyph = codepoint - 32u;
    let row = font_atlas[glyph * 8u + cell_y];
    let bit = (row >> (7u - cell_x)) & 1u;

    return f32(bit);
}

fn draw_line_number(line_number: u32, digit_index: u32, cell_x: u32, cell_y: u32) -> f32 {
    if digit_index >= GUTTER_COLS {
        return 0.0;
    }

    var divisor: u32 = 1u;
    switch digit_index {
        case 0u: {
            divisor = 1000u;
        }
        case 1u: {
            divisor = 100u;
        }
        case 2u: {
            divisor = 10u;
        }
        default: {
            divisor = 1u;
        }
    }

    let digit = (line_number / divisor) % 10u;
    return sample_glyph(48u + digit, cell_x, cell_y);
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
    );

    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, 0.0),
    );

    var out: VertexOutput;
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.uv = uvs[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_x = in.uv.x * CHAR_WIDTH * f32(COLS_VISIBLE);
    let screen_y = in.uv.y * CHAR_HEIGHT * f32(ROWS_VISIBLE);

    let col = u32(floor(screen_x / CHAR_WIDTH));
    let row = u32(floor(screen_y / CHAR_HEIGHT));

    let cell_x = u32(screen_x) % 8u;
    let cell_y = u32(screen_y) % 8u;

    let scroll_line = atomicLoad(&state.scroll_line);
    let scroll_col = atomicLoad(&state.scroll_col);

    let world_line = row + scroll_line;
    let world_col = col + scroll_col;

    if col < GUTTER_COLS {
        let value = draw_line_number(world_line + 1u, col, cell_x, cell_y);
        let gutter_color = mix(COLOR_BG, COLOR_GUTTER, value);
        return vec4<f32>(gutter_color, 1.0);
    }

    let text_col = world_col - GUTTER_COLS;

    let cursor_line = atomicLoad(&state.cursor_line);
    let cursor_col = atomicLoad(&state.cursor_col);

    if world_line == cursor_line && text_col == cursor_col {
        let frame = atomicLoad(&state.frame_count);
        let blink = (frame / 30u) % 2u;
        if blink == 1u {
            return vec4<f32>(COLOR_CURSOR, 1.0);
        }
    }

    let codepoint = get_char_at(world_line, text_col);
    let glyph_value = sample_glyph(codepoint, cell_x, cell_y);
    let color = mix(COLOR_BG, COLOR_FG, glyph_value);

    return vec4<f32>(color, 1.0);
}
