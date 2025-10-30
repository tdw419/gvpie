// ============================================================================
// GVPIE Canonical I/O Contract - WGSL Mirror
// ============================================================================
// THIS FILE MUST MATCH `src/io_contract.rs` EXACTLY.
// Import in shaders with: #import "contract.wgsl"

// ============================================================================
// BUFFER LAYOUTS (Must match Rust exactly)
// ============================================================================

struct EditorState {
    text_data: array<u32, 262144>,
    gap_start: u32,
    gap_end: u32,
    total_chars: u32,
    cursor_pos: u32,
    dirty: u32,

    line_offsets: array<u32, 65536>,
    line_count: u32,
    lines_dirty: u32,

    cursor_line: u32,
    cursor_col: u32,
    scroll_offset: u32,
    selection_start: u32,
    selection_end: u32,
}

struct RenderUniforms {
    time: f32,
    viewport_width: f32,
    viewport_height: f32,
    _padding: f32,
}

struct Camera3D {
    position: vec3<f32>,
    focus: vec3<f32>,
    up: vec3<f32>,
    fov: f32,
    move_speed: f32,
    mouse_sensitivity: f32,
}

// ============================================================================
// EVENT SYSTEM (Must match Rust exactly)
// ============================================================================

const EVENT_NONE: u32 = 0u;
const EVENT_CHARACTER: u32 = 1u;
const EVENT_SPECIAL_KEY: u32 = 2u;
const EVENT_MOUSE_MOVE: u32 = 3u;
const EVENT_MOUSE_BUTTON: u32 = 4u;
const EVENT_SCROLL: u32 = 5u;

const KEY_BACKSPACE: u32 = 8u;
const KEY_TAB: u32 = 9u;
const KEY_ENTER: u32 = 13u;
const KEY_SHIFT: u32 = 16u;
const KEY_CTRL: u32 = 17u;
const KEY_ALT: u32 = 18u;
const KEY_ESCAPE: u32 = 27u;
const KEY_SPACE: u32 = 32u;
const KEY_LEFT: u32 = 37u;
const KEY_UP: u32 = 38u;
const KEY_RIGHT: u32 = 39u;
const KEY_DOWN: u32 = 40u;
const KEY_A: u32 = 0x0004u;
const KEY_D: u32 = 0x0007u;
const KEY_S: u32 = 0x0016u;
const KEY_W: u32 = 0x001Au;
const KEY_DELETE: u32 = 127u;

const MOD_CTRL: u32 = 1u;
const MOD_SHIFT: u32 = 2u;
const MOD_ALT: u32 = 4u;

// ============================================================================
// BINDING LAYOUT (Must match Rust exactly)
// ============================================================================

const BINDING_GROUP: u32 = 0u;
const BINDING_STATE: u32 = 0u;
const BINDING_UNIFORMS: u32 = 1u;
const BINDING_EVENTS: u32 = 2u;
const BINDING_REQUESTS: u32 = 3u;
const BINDING_FONT_TEXTURE: u32 = 2u;
const BINDING_FONT_SAMPLER: u32 = 3u;

// ============================================================================
// 3D MODE & CAMERA SYSTEM (Must match Rust exactly)
// ============================================================================

const MODE_2D: u32 = 0u;
const MODE_3D: u32 = 10001u;

const CAMERA_DATA_OFFSET: u32 = 0u;
const CAMERA_DATA_FLOATS: u32 = 16u;
const CAMERA_SENTINEL_INDEX: u32 = CAMERA_DATA_OFFSET + CAMERA_DATA_FLOATS - 1u;
const CAMERA_SENTINEL: u32 = 0xC0FFEE00u;
const DEG_TO_RAD: f32 = 0.017453292519943295;

fn is_3d_mode(scroll_offset: u32) -> bool {
    return scroll_offset >= MODE_3D;
}

fn bitcast_f32(value: u32) -> f32 {
    return bitcast<f32>(value);
}

fn bitcast_u32(value: f32) -> u32 {
    return bitcast<u32>(value);
}

fn default_camera_3d() -> Camera3D {
    return Camera3D(
        vec3<f32>(0.0, 5.0, 10.0),
        vec3<f32>(0.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        60.0 * DEG_TO_RAD,
        5.0,
        0.002,
    );
}

fn load_camera_3d() -> Camera3D {
    let base = CAMERA_DATA_OFFSET;
    return Camera3D(
        vec3<f32>(
            bitcast_f32(state.text_data[base + 0u]),
            bitcast_f32(state.text_data[base + 1u]),
            bitcast_f32(state.text_data[base + 2u])
        ),
        vec3<f32>(
            bitcast_f32(state.text_data[base + 3u]),
            bitcast_f32(state.text_data[base + 4u]),
            bitcast_f32(state.text_data[base + 5u])
        ),
        vec3<f32>(
            bitcast_f32(state.text_data[base + 6u]),
            bitcast_f32(state.text_data[base + 7u]),
            bitcast_f32(state.text_data[base + 8u])
        ),
        bitcast_f32(state.text_data[base + 9u]),
        bitcast_f32(state.text_data[base + 10u]),
        bitcast_f32(state.text_data[base + 11u])
    );
}

fn camera_is_initialised() -> bool {
    return state.text_data[CAMERA_SENTINEL_INDEX] == CAMERA_SENTINEL;
}

fn get_camera_ray(camera: Camera3D, frag_coord: vec2<f32>, viewport: vec2<f32>) -> vec3<f32> {
    let forward = normalize(camera.focus - camera.position);
    let right = normalize(cross(forward, camera.up));
    let up_vec = cross(right, forward);

    let aspect = viewport.x / viewport.y;
    let tan_fov = tan(camera.fov * 0.5);

    let uv = (frag_coord / viewport) * 2.0 - 1.0;
    let dir = normalize(
        forward +
        right * uv.x * aspect * tan_fov +
        up_vec * uv.y * tan_fov
    );
    return dir;
}

// ============================================================================
// CARD / GAUSSIAN SPLAT SYSTEM
// ============================================================================

struct SplatData {
    position: vec3<f32>,
    scale: f32,
    color: vec3<f32>,
    alpha: f32,
}

const CARDS_SENTINEL: u32 = 0xDEC0DE00u;
const CARDS_META_FLOATS: u32 = 4u;
const SPLAT_FLOATS: u32 = 8u;
const MAX_SPLATS: u32 = 512u;
const CARDS_DATA_OFFSET: u32 = CAMERA_DATA_OFFSET + CAMERA_DATA_FLOATS;
const CARDS_SENTINEL_INDEX: u32 = CARDS_DATA_OFFSET;
const CARDS_COUNT_INDEX: u32 = CARDS_SENTINEL_INDEX + 1u;
const CARDS_SELECTED_INDEX: u32 = CARDS_SENTINEL_INDEX + 2u;
const CARDS_HOVERED_INDEX: u32 = CARDS_SENTINEL_INDEX + 3u;
const CARDS_TIME_INDEX: u32 = CARDS_SENTINEL_INDEX + 4u;
const CARDS_DATA_START: u32 = CARDS_SENTINEL_INDEX + 5u;
const CARDS_TOTAL_FLOATS: u32 = MAX_SPLATS * SPLAT_FLOATS;

fn cards_initialized() -> bool {
    return state.text_data[CARDS_SENTINEL_INDEX] == CARDS_SENTINEL;
}

fn get_cards_count() -> u32 {
    return state.text_data[CARDS_COUNT_INDEX];
}

fn get_cards_time() -> f32 {
    return bitcast_f32(state.text_data[CARDS_TIME_INDEX]);
}

fn read_splat(index: u32) -> SplatData {
    let base = CARDS_DATA_START + index * SPLAT_FLOATS;
    return SplatData(
        vec3<f32>(
            bitcast_f32(state.text_data[base + 0u]),
            bitcast_f32(state.text_data[base + 1u]),
            bitcast_f32(state.text_data[base + 2u])
        ),
        bitcast_f32(state.text_data[base + 3u]),
        vec3<f32>(
            bitcast_f32(state.text_data[base + 4u]),
            bitcast_f32(state.text_data[base + 5u]),
            bitcast_f32(state.text_data[base + 6u])
        ),
        bitcast_f32(state.text_data[base + 7u])
    );
}

fn evaluate_gaussian_splat(splat: SplatData, point: vec3<f32>) -> f32 {
    let delta = point - splat.position;
    let scaled = delta / splat.scale;
    let distance_sq = dot(scaled, scaled);
    return exp(-distance_sq * 0.5) * splat.alpha;
}
